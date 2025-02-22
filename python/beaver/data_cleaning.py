import polars as pl


class DataCleaning:
    def clean_non_numeric_entries(
        self, csv_file: str, column_name: str
    ) -> pl.DataFrame:
        """
        Reads a CSV file, replaces non-numeric values in the specified column with empty strings.

        :param csv_file: Path to the CSV file
        :param column_name: Name of the column to clean
        :return: Cleaned Polars DataFrame
        """
        return self.replace_entries_with_empty(csv_file, column_name, r"[^0-9.]")

    def replace_entries_with_empty(self, csv_file: str, column_name: str, pattern: str):
        """
        Reads a CSV file and replaces values matching the pattern with an empty string.

        :param csv_file: Path to the CSV file
        :param column_name: Name of the column to modify
        :param pattern: Regex pattern to match and remove
        :return: Modified Polars DataFrame
        """
        df = pl.read_csv(csv_file)
        df = df.with_columns(df[column_name].cast(pl.Utf8).str.replace_all(pattern, ""))
        return df

    def separated_with(
        self, csv_file: str, column_name: str, separator: str
    ) -> pl.DataFrame:
        """
        Reads a CSV file and splits the values in the specified column using the given separator.

        :param csv_file: Path to the CSV file
        :param column_name: Name of the column to split
        :param separator: The character or string used as the separator
        :return: Modified Polars DataFrame with split values as lists
        """
        df = pl.read_csv(csv_file)
        df = df.with_columns(df[column_name].cast(pl.Utf8).str.split(separator))
        return df
