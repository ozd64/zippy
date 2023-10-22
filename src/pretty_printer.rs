use crate::zip::Zip;

const COLUMNS: [&'static str; 4] = ["Size (Bytes)", "Date Time", "Environment", "Name"];

pub fn pretty_print_zip_files(zip: &Zip) {
    println!(
        "\nFile Count: {}, Directory Count: {}\n",
        zip.file_count(),
        zip.dir_count()
    );

    println!("{}\t{}\t\t{}\t{}", COLUMNS[0], COLUMNS[1], COLUMNS[2], COLUMNS[3]);

    let column_separator_1 = String::from_utf8(vec![b'-'; COLUMNS[0].len()]).unwrap();
    let column_separator_2 = String::from_utf8(vec![b'-'; 19]).unwrap();
    let column_separator_3 = String::from_utf8(vec![b'-'; 12]).unwrap();
    let column_separator_4 = String::from_utf8(vec![b'-'; 20]).unwrap();

    println!("{}\t{}\t{}\t{}", column_separator_1, column_separator_2, column_separator_3, column_separator_4);

    zip.zip_files().iter().for_each(|zip_file| {
        let first_column_padding =
            COLUMNS[0].len() - zip_file.uncompressed_size().to_string().len();

        println!(
            "{}{}\t{}\t{}\t\t{}",
            String::from_utf8(vec![b' '; first_column_padding]).unwrap(),
            zip_file.uncompressed_size(),
            zip_file.date_time(),
            zip_file.environment(),
            zip_file.file_name()
        );
    });
}
