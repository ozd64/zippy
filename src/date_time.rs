use std::fmt::Display;

struct ZipDate {
    day: u8,
    month: u8,
    year: u16,
}

struct ZipTime {
    hour: u8,
    min: u8,
    second: u8,
}

pub struct ZipDateTime {
    date: ZipDate,
    time: ZipTime,
}

impl ZipDateTime {
    pub fn from_bytes(date: u16, time: u16) -> ZipDateTime {
        let day = (date & 0x001F) as u8;
        let month = ((date >> 5) & 0x000F) as u8;
        let year = (date >> 9) + 1980;

        let date = ZipDate { day, month, year };

        let second = ((time & 0x001F) * 2) as u8;
        let min = ((time >> 5) & 0x003F) as u8;
        let hour = (time >> 11) as u8;

        let time = ZipTime { hour, min, second };

        ZipDateTime { date, time }
    }
}

impl Display for ZipDateTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:02}/{}/{} {}:{}:{}",
            self.date.month,
            self.date.day,
            self.date.year,
            self.time.hour,
            self.time.min,
            self.time.second
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_date_time() {
        let time = 0xA76F;
        let date = 0x5739;

        let zip_date_time = ZipDateTime::from_bytes(date, time);

        assert_eq!(zip_date_time.date.day, 25);
        assert_eq!(zip_date_time.date.month, 9);
        assert_eq!(zip_date_time.date.year, 2023);

        assert_eq!(zip_date_time.time.hour, 20);
        assert_eq!(zip_date_time.time.min, 59);
        assert_eq!(zip_date_time.time.second, 30);
    }
}
