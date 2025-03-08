use std::{
    collections::{HashMap, HashSet},
    sync::{
        mpsc::{Receiver, Sender},
        Arc, Mutex,
    },
};

use crate::{
    distances::CachedDistanceCalculator,
    frame::Frame,
    id::ID,
    trackers::{RecordScore, Tracker},
};

/// Tracking worker command
///
/// Represents a command that can be sent to the tracking worker.
pub enum TrackingWorkerCommand {
    /// Add trackers to be managed by the worker
    AddTrackers(HashMap<ID, Arc<Mutex<Tracker>>>),
    /// Remove trackers from the worker
    RemoveTrackers(Vec<ID>),
    /// Process a frame
    ProcessFrame(usize),
}

/// Tracking worker response
///
/// Represents a response that the tracking worker can send back.
pub enum TrackingWorkerResponse {
    /// Scores of the processed frame
    ProcessFrame(HashMap<ID, Vec<RecordScore>>),
}

/// Tracking worker handler
///
/// Creating a new handler will spawn a new thread.
///
/// It is responsible of the communication with the tracking worker.
/// It sends commands to the worker and waits for responses from the worker
/// in the other thread.
///
/// It also keeps track the IDs of the trackers it manages, it is useful
/// for the main thread to know how many trackers are being managed by this worker.
pub struct TrackingWorkerHandler {
    sender: Sender<TrackingWorkerCommand>,
    receiver: Receiver<TrackingWorkerResponse>,
    tracker_ids: HashSet<ID>,
}

impl TrackingWorkerHandler {
    pub fn new(
        frames: Arc<Vec<Frame>>,
        trackers: HashMap<ID, Arc<Mutex<Tracker>>>,
        distance_calculators: Vec<CachedDistanceCalculator>,
    ) -> Self {
        let (sender_cmd, receiver_cmd) = std::sync::mpsc::channel();
        let (sender_resp, receiver_resp) = std::sync::mpsc::channel();
        let tracker_ids = trackers.keys().cloned().collect();

        std::thread::spawn(move || {
            let mut worker = TrackingWorker::new(
                receiver_cmd,
                sender_resp,
                frames,
                trackers,
                distance_calculators,
            );
            worker.run();
        });

        Self {
            sender: sender_cmd,
            receiver: receiver_resp,
            tracker_ids,
        }
    }

    /// Returns the number of trackers managed by this worker
    pub fn num_trackers(&self) -> usize {
        self.tracker_ids.len()
    }

    /// Adds trackers to be managed by this worker
    pub fn add_trackers(&mut self, trackers: HashMap<ID, Arc<Mutex<Tracker>>>) {
        for id in trackers.keys() {
            self.tracker_ids.insert(*id);
        }

        self.sender
            .send(TrackingWorkerCommand::AddTrackers(trackers))
            .unwrap();
    }

    /// Removes trackers from this worker
    pub fn remove_trackers(&mut self, ids: Vec<ID>) {
        for id in ids.iter() {
            self.tracker_ids.remove(id);
        }

        self.sender
            .send(TrackingWorkerCommand::RemoveTrackers(ids))
            .unwrap();
    }

    pub fn process_frame(&self, frame_idx: usize) {
        self.sender
            .send(TrackingWorkerCommand::ProcessFrame(frame_idx))
            .unwrap();
    }

    /// Waits for the scores of the processed frame
    ///
    /// This should be called after `process_frame`, it is the equivalent of
    /// a join.
    pub fn wait_scores(&self) -> HashMap<ID, Vec<RecordScore>> {
        match self.receiver.recv() {
            Ok(TrackingWorkerResponse::ProcessFrame(scores)) => scores,
            _ => panic!("invalid response"),
        }
    }
}

/// Tracking worker
///
/// Responsible for processing the frames for the trackers it manages.
/// It executes in a separate thread and communicates with the main thread
/// through channels.
///
/// The worker itself lives in the other thread and processes the commands
/// it receives from its handler in the main thread.
pub struct TrackingWorker {
    receiver: Receiver<TrackingWorkerCommand>,
    sender: Sender<TrackingWorkerResponse>,
    frames: Arc<Vec<Frame>>,
    trackers: HashMap<ID, Arc<Mutex<Tracker>>>,
    distance_calculators: Vec<CachedDistanceCalculator>,
}

impl TrackingWorker {
    pub fn new(
        receiver: Receiver<TrackingWorkerCommand>,
        sender: Sender<TrackingWorkerResponse>,
        frames: Arc<Vec<Frame>>,
        trackers: HashMap<ID, Arc<Mutex<Tracker>>>,
        distance_calculators: Vec<CachedDistanceCalculator>,
    ) -> Self {
        Self {
            receiver,
            sender,
            frames,
            trackers,
            distance_calculators,
        }
    }

    /// Main loop of the worker
    ///
    /// Waits for commands from the main thread and processes them.
    pub fn run(&mut self) {
        loop {
            match self.receiver.recv() {
                Ok(TrackingWorkerCommand::AddTrackers(trackers)) => {
                    self.add_trackers(trackers);
                }
                Ok(TrackingWorkerCommand::RemoveTrackers(ids)) => {
                    self.remove_trackers(ids);
                }
                Ok(TrackingWorkerCommand::ProcessFrame(frame_idx)) => {
                    let scores = self.process_frame(frame_idx);
                    self.sender
                        .send(TrackingWorkerResponse::ProcessFrame(scores))
                        .unwrap();
                }
                Err(_) => return,
            }
        }
    }

    fn add_trackers(&mut self, trackers: HashMap<ID, Arc<Mutex<Tracker>>>) {
        self.trackers.extend(trackers);
    }

    fn remove_trackers(&mut self, ids: Vec<ID>) {
        for id in ids {
            self.trackers.remove(&id);
        }
    }

    fn process_frame(&mut self, frame_idx: usize) -> HashMap<ID, Vec<RecordScore>> {
        let frame = &self.frames[frame_idx];

        let mut trackers_scores = HashMap::with_capacity(self.trackers.len());

        for (_, tracker) in self.trackers.iter_mut() {
            let mut tracker = tracker.lock().unwrap();
            let scores = tracker.process_frame(frame, &mut self.distance_calculators);

            trackers_scores.insert(tracker.id(), scores);
        }

        trackers_scores
    }
}
