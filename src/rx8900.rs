use heapless::Vec;
use chrono::{
    Datelike, NaiveDate, NaiveDateTime, NaiveTime, Timelike, Weekday,
};
use embedded_hal::blocking::i2c::{Read, Write, WriteRead};

const RX8900_ADDR: u8 = 0x32;

#[derive(Clone, Copy, Debug, PartialEq)]
enum RegisterTable {
    // Compatible with the RX-8803
    CompatibleSEC = 0x00, // Seconds
    CompatibleMIN = 0x01, // Minutes
    CompatibleHOUR = 0x02, // Hours
    CompatibleWEEK = 0x03, // Day of the week
    CompatibleDAY = 0x04, // Day
    CompatibleMONTH = 0x05, // Month
    CompatibleYEAR = 0x06, // Year
    CompatibleRAM = 0x07, // RAM
    CompatibleMinAlarm = 0x08, // Minutes alarm
    CompatibleHourAlarm = 0x09, // Hours alarm
    CompatibleWeekDayAlarm = 0x0A, // Day of the week alarm
    CompatibleTimerCounter0 = 0x0B, // Timer/Counter 0
    CompatibleTimerCounter1 = 0x0C, // Timer/Counter 1
    CompatibleExtensionRegister = 0x0D,
    CompatibleFlagRegister = 0x0E,
    CompatibleControlRegister = 0x0F,

    // Extended
    ExtendedSEC = 0x10, // Seconds
    ExtendedMIN = 0x11, // Minutes
    ExtendedHOUR = 0x12, // Hours
    ExtendedWEEK = 0x13, // Day of the week
    ExtendedDAY = 0x14, // Day
    ExtendedMONTH = 0x15, // Month
    ExtendedYEAR = 0x16, // Year
    ExtendedTEMP = 0x17, // TEMP
    ExtendedBackupFunction = 0x18, // Backup function
    ExtendedTimerCounter0 = 0x1B, // Timer/Counter 0
    ExtendedTimerCounter1 = 0x1C, // Timer/Counter 1
    ExtendedExtensionRegister = 0x1D,
    ExtendedFlagRegister = 0x1E,
    ExtendedControlRegister = 0x1F,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SourceClock {
    SourceClock4096Hz = 0b00,
    SourceClock64Hz = 0b01,
    SourceClockSecond = 0b10,
    SourceClockMinute = 0b11,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AlarmType {
    WeekAlarm = 0b00,
    DayAlarm = 0b01,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum UpdateInterruptType {
    EverySecond = 0b00,
    EveryMinute = 0b01,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FoutFrequency {
    FoutFrequency32_768kHz = 0b00,
    FoutFrequency1024Hz = 0b01,
    FoutFrequency1Hz = 0b10,
    // FoutFrequency32_768kHz = 0b11,
}


#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CompensationIntervalType {
    CompensationInterval0_5s = 0b00,
    CompensationInterval2_0s = 0b01,
    CompensationInterval10s = 0b10,
    CompensationInterval30s = 0b11,
}
pub struct Rx8900<I2C> {
    i2c: I2C,
}

impl<I2C> Rx8900<I2C> {
    pub fn new(i2c: I2C) -> Self {
        Self { i2c, }
    }

    fn from_bcd(data: u8) -> u8 {
        (data >> 4) * 10 + (data & 0x0F)
    }

    fn to_bcd(data: u8) -> u8 {
        ((data / 10) << 4) | (data % 10)
    }

    fn from_week(data: u8) -> Weekday {
        match data {
            0b00000001 => Weekday::Sun,
            0b00000010 => Weekday::Mon,
            0b00000100 => Weekday::Tue,
            0b00001000 => Weekday::Wed,
            0b00010000 => Weekday::Thu,
            0b00100000 => Weekday::Fri,
            0b01000000 => Weekday::Sat,
            _ => todo!(),
        }
    }

    fn to_week(data: Weekday) -> u8 {
        match data {
            Weekday::Sun => 0b00000001,
            Weekday::Mon => 0b00000010,
            Weekday::Tue => 0b00000100,
            Weekday::Wed => 0b00001000,
            Weekday::Thu => 0b00010000,
            Weekday::Fri => 0b00100000,
            Weekday::Sat => 0b01000000,
        }
    }
}

impl<I2C, E> Rx8900<I2C>
where
    I2C: Read<Error = E> + WriteRead<Error = E>,
{
    fn read_register(&mut self, register: RegisterTable) -> Result<u8, E> {
        let mut data = [0];
        self.i2c.write_read(RX8900_ADDR, &[register as u8], &mut data)?;
        Ok(data[0])
    }

    fn read_register_1bit(&mut self, register: RegisterTable, bit: u8) -> Result<bool, E> {
        let data = self.read_register(register)?;
        Ok((data & (1 << bit)) == (1 << bit))
    }

    pub fn sec(&mut self) -> Result<u8, E> {
        let data = self.read_register(RegisterTable::CompatibleSEC)?;
        Ok(Self::from_bcd(data & 0b01111111))
    }

    pub fn min(&mut self) -> Result<u8, E> {
        let data = self.read_register(RegisterTable::CompatibleMIN)?;
        Ok(Self::from_bcd(data & 0b01111111))
    }

    pub fn hour(&mut self) -> Result<u8, E> {
        let data = self.read_register(RegisterTable::CompatibleHOUR)?;
        Ok(Self::from_bcd(data & 0b00111111))
    }

    pub fn week(&mut self) -> Result<Weekday, E> {
        let data = self.read_register(RegisterTable::CompatibleWEEK)?;
        Ok(Self::from_week(data))
    }

    pub fn day(&mut self) -> Result<u8, E> {
        let data = self.read_register(RegisterTable::CompatibleDAY)?;
        Ok(Self::from_bcd(data & 0b00111111))
    }

    pub fn month(&mut self) -> Result<u8, E> {
        let data = self.read_register(RegisterTable::CompatibleMONTH)?;
        Ok(Self::from_bcd(data & 0b00011111))
    }

    pub fn year(&mut self) -> Result<u8, E> {
        let data = self.read_register(RegisterTable::CompatibleYEAR)?;
        Ok(Self::from_bcd(data))
    }

    pub fn ram(&mut self) -> Result<u8, E> {
        self.read_register(RegisterTable::CompatibleRAM)
    }

    pub fn min_alarm(&mut self) -> Result<u8, E> {
        let data = self.read_register(RegisterTable::CompatibleMinAlarm)?;
        Ok(Self::from_bcd(data & 0b01111111))
    }

    pub fn min_alarm_enabled(&mut self) -> Result<bool, E> {
        self.read_register_1bit(RegisterTable::CompatibleMinAlarm, 7)
    }

    pub fn hour_alarm(&mut self) -> Result<u8, E> {
        let data = self.read_register(RegisterTable::CompatibleHourAlarm)?;
        Ok(Self::from_bcd(data & 0b00111111))
    }

    pub fn hour_alarm_enabled(&mut self) -> Result<bool, E> {
        self.read_register_1bit(RegisterTable::CompatibleHourAlarm, 7)
    }

    pub fn week_alarm(&mut self) -> Result<Vec<Weekday, 7>, E> {
        let mut weekdays = Vec::<Weekday, 7>::new();
        let data = self.read_register(RegisterTable::CompatibleWeekDayAlarm)?;
        for i in 0..7 {
            let bit = 1 << i;
            if (data & bit) == bit {
                weekdays.push(Self::from_week(bit)).unwrap();
            }
        }
        Ok(weekdays)
    }

    pub fn week_alarm_enabled(&mut self) -> Result<bool, E> {
        self.read_register_1bit(RegisterTable::CompatibleWeekDayAlarm, 7)
    }

    pub fn day_alarm(&mut self) -> Result<u8, E> {
        let data = self.read_register(RegisterTable::CompatibleWeekDayAlarm)?;
        Ok(Self::from_bcd(data & 0b01111111))
    }

    pub fn day_alarm_enabled(&mut self) -> Result<bool, E> {
        self.read_register_1bit(RegisterTable::CompatibleWeekDayAlarm, 7)
    }

    pub fn timer_counter0(&mut self) -> Result<u8, E> {
        self.read_register(RegisterTable::CompatibleTimerCounter0)
    }

    pub fn timer_counter1(&mut self) -> Result<u8, E> {
        self.read_register(RegisterTable::CompatibleTimerCounter1)
    }
    
    pub fn timer_counter(&mut self) -> Result<u16, E> {
        let data0 = self.read_register(RegisterTable::CompatibleTimerCounter0)?;
        let data1 = self.read_register(RegisterTable::CompatibleTimerCounter1)?;
        Ok((data1 as u16) << 8 | data0 as u16)
    }

    pub fn temp(&mut self) -> Result<u8, E> {
        self.read_register(RegisterTable::ExtendedTEMP)
    }

    pub fn temp_in_cercius(&mut self) -> Result<f32, E> {
        let data = self.temp()?;
        Ok((data as f32 * 2.0 - 187.19) / 3.218)
    }

    pub fn vdetoff(&mut self) -> Result<bool, E> {
        self.read_register_1bit(RegisterTable::ExtendedBackupFunction, 3)
    }

    pub fn voltage_detector_off(&mut self) -> Result<bool, E> {
        self.vdetoff()
    }

    pub fn swoff(&mut self) -> Result<bool, E> {
        self.read_register_1bit(RegisterTable::ExtendedBackupFunction, 2)
    }

    pub fn switch_off(&mut self) -> Result<bool, E> {
        self.swoff()
    }

    pub fn bksmp1(&mut self) -> Result<bool, E> {
        self.read_register_1bit(RegisterTable::ExtendedBackupFunction, 1)
    }

    pub fn bksmp0(&mut self) -> Result<bool, E> {
        self.read_register_1bit(RegisterTable::ExtendedBackupFunction, 0)
    }

    pub fn bksmp(&mut self) -> Result<u8, E> {
        let data = self.read_register(RegisterTable::ExtendedBackupFunction)?;
        Ok((data & 0b00000011) >> 0)
    }

    pub fn backup_mode_sampling_time(&mut self) -> Result<u8, E> {
        self.bksmp()
    }

    pub fn alarm_type(&mut self) -> Result<AlarmType, E> {
        self.wada().map(|x| if x { AlarmType::DayAlarm } else { AlarmType::WeekAlarm })
    }

    pub fn test(&mut self) -> Result<bool, E> {
        self.read_register_1bit(RegisterTable::CompatibleExtensionRegister, 7)
    }

    pub fn wada(&mut self) -> Result<bool, E> {
        self.read_register_1bit(RegisterTable::CompatibleExtensionRegister, 6)
    }

    pub fn update_interrupt_type(&mut self) -> Result<UpdateInterruptType, E> {
        self.usel().map(|x| if x { UpdateInterruptType::EveryMinute } else { UpdateInterruptType::EverySecond })
    }

    pub fn usel(&mut self) -> Result<bool, E> {
        self.read_register_1bit(RegisterTable::CompatibleExtensionRegister, 5)
    }

    pub fn te(&mut self) -> Result<bool, E> {
        self.read_register_1bit(RegisterTable::CompatibleExtensionRegister, 4)
    }

    pub fn timer_enable(&mut self) -> Result<bool, E> {
        self.te()
    }

    pub fn fsel1(&mut self) -> Result<bool, E> {
        self.read_register_1bit(RegisterTable::CompatibleExtensionRegister, 3)
    }

    pub fn fsel0(&mut self) -> Result<bool, E> {
        self.read_register_1bit(RegisterTable::CompatibleExtensionRegister, 2)
    }

    pub fn fsel(&mut self) -> Result<u8, E> {
        let data = self.read_register(RegisterTable::CompatibleExtensionRegister)?;
        Ok((data & 0b00001100) >> 2)
    }

    pub fn fout_frequency(&mut self) -> Result<FoutFrequency, E> {
        let current = self.fsel()?;
        Ok(match current {
            0b00 => FoutFrequency::FoutFrequency32_768kHz,
            0b01 => FoutFrequency::FoutFrequency1024Hz,
            0b10 => FoutFrequency::FoutFrequency1Hz,
            0b11 => FoutFrequency::FoutFrequency32_768kHz,
            _ => todo!(),
        })
    }

    pub fn source_clock(&mut self) -> Result<SourceClock, E> {
        let data = self.read_register(RegisterTable::CompatibleExtensionRegister)?;
        Ok(match (data & 0b00000011) >> 0 {
            0b00 => SourceClock::SourceClock4096Hz,
            0b01 => SourceClock::SourceClock64Hz,
            0b10 => SourceClock::SourceClockSecond,
            0b11 => SourceClock::SourceClockMinute,
            _ => todo!(),
        })
    }
    
    pub fn tsel1(&mut self) -> Result<bool, E> {
        self.read_register_1bit(RegisterTable::CompatibleExtensionRegister, 1)
    }

    pub fn tsel0(&mut self) -> Result<bool, E> {
        self.read_register_1bit(RegisterTable::CompatibleExtensionRegister, 0)
    }

    pub fn tsel(&mut self) -> Result<u8, E> {
        let data = self.read_register(RegisterTable::CompatibleExtensionRegister)?;
        Ok((data & 0b00000011) >> 0)
    }

    pub fn uf(&mut self) -> Result<bool, E> {
        self.read_register_1bit(RegisterTable::CompatibleFlagRegister, 5)
    }

    pub fn update_flag(&mut self) -> Result<bool, E> {
        self.uf()
    }

    pub fn tf(&mut self) -> Result<bool, E> {
        self.read_register_1bit(RegisterTable::CompatibleFlagRegister, 4)
    }

    pub fn timer_flag(&mut self) -> Result<bool, E> {
        self.tf()
    }

    pub fn af(&mut self) -> Result<bool, E> {
        self.read_register_1bit(RegisterTable::CompatibleFlagRegister, 3)
    }

    pub fn alarm_flag(&mut self) -> Result<bool, E> {
        self.af()
    }

    pub fn vlf(&mut self) -> Result<bool, E> {
        self.read_register_1bit(RegisterTable::CompatibleFlagRegister, 1)
    }

    pub fn voltage_low_flag(&mut self) -> Result<bool, E> {
        self.vlf()
    }

    pub fn vdet(&mut self) -> Result<bool, E> {
        self.read_register_1bit(RegisterTable::CompatibleFlagRegister, 0)
    }

    pub fn voltage_detect_flag(&mut self) -> Result<bool, E> {
        self.vdet()
    }

    pub fn csel1(&mut self) -> Result<bool, E> {
        self.read_register_1bit(RegisterTable::CompatibleControlRegister, 7)
    }

    pub fn csel0(&mut self) -> Result<bool, E> {
        self.read_register_1bit(RegisterTable::CompatibleControlRegister, 6)
    }

    pub fn csel(&mut self) -> Result<u8, E> {
        let data = self.read_register(RegisterTable::CompatibleControlRegister)?;
        Ok((data & 0b11000000) >> 6)
    }

    pub fn compensation_interval_type(&mut self) -> Result<CompensationIntervalType, E> {
        let current = self.csel()?;
        Ok(match current {
            0b00 => CompensationIntervalType::CompensationInterval0_5s,
            0b01 => CompensationIntervalType::CompensationInterval2_0s,
            0b10 => CompensationIntervalType::CompensationInterval10s,
            0b11 => CompensationIntervalType::CompensationInterval30s,
            _ => todo!(),
        })
    }

    pub fn uie(&mut self) -> Result<bool, E> {
        self.read_register_1bit(RegisterTable::CompatibleControlRegister, 5)
    }

    pub fn update_interrupt_enable(&mut self) -> Result<bool, E> {
        self.uie()
    }

    pub fn tie(&mut self) -> Result<bool, E> {
        self.read_register_1bit(RegisterTable::CompatibleControlRegister, 4)
    }

    pub fn timer_interrupt_enable(&mut self) -> Result<bool, E> {
        self.tie()
    }

    pub fn aie(&mut self) -> Result<bool, E> {
        self.read_register_1bit(RegisterTable::CompatibleControlRegister, 3)
    }

    pub fn alarm_interrupt_enable(&mut self) -> Result<bool, E> {
        self.aie()
    }

    pub fn reset(&mut self) -> Result<bool, E> {
        self.read_register_1bit(RegisterTable::CompatibleControlRegister, 0)
    }
}

impl<I2C, E> Rx8900<I2C>
where
    I2C: Write<Error = E>,
{
    fn write_register(&mut self, register: RegisterTable, data: u8) -> Result<(), E> {
        self.i2c.write(RX8900_ADDR, &[register as u8, data])?;
        Ok(())
    }

    pub fn set_sec(&mut self, data: u8) -> Result<(), E> {
        self.write_register(RegisterTable::CompatibleSEC, Self::to_bcd(data & 0b01111111))
    }

    pub fn set_min(&mut self, data: u8) -> Result<(), E> {
        self.write_register(RegisterTable::CompatibleMIN, Self::to_bcd(data & 0b01111111))
    }

    pub fn set_hour(&mut self, data: u8) -> Result<(), E> {
        self.write_register(RegisterTable::CompatibleHOUR, Self::to_bcd(data & 0b00111111))
    }

    pub fn set_week(&mut self, data: Weekday) -> Result<(), E> {
        self.write_register(RegisterTable::CompatibleWEEK, Self::to_week(data))
    }

    pub fn set_day(&mut self, data: u8) -> Result<(), E> {
        self.write_register(RegisterTable::CompatibleDAY, Self::to_bcd(data & 0b00111111))
    }

    pub fn set_month(&mut self, data: u8) -> Result<(), E> {
        self.write_register(RegisterTable::CompatibleMONTH, Self::to_bcd(data & 0b00011111))
    }

    pub fn set_year(&mut self, data: u8) -> Result<(), E> {
        self.write_register(RegisterTable::CompatibleYEAR, Self::to_bcd(data))
    }

    pub fn set_ram(&mut self, data: u8) -> Result<(), E> {
        self.write_register(RegisterTable::CompatibleRAM, data)
    }

    pub fn set_min_alarm(&mut self, data: u8, enabled: bool) -> Result<(), E> {
        self.write_register(RegisterTable::CompatibleMinAlarm, Self::to_bcd(data & 0b01111111) | (enabled as u8) << 7)
    }

    pub fn set_hour_alarm(&mut self, data: u8, enabled: bool) -> Result<(), E> {
        self.write_register(RegisterTable::CompatibleHourAlarm, Self::to_bcd(data & 0b00111111) | (enabled as u8) << 7)
    }

    pub fn set_week_alarm(&mut self, data: &Vec<Weekday, 7>) -> Result<(), E> {
        let mut value = 0;
        for day in data {
            value |= Self::to_week(*day);
        }
        self.write_register(RegisterTable::CompatibleWeekDayAlarm, value)
    }

    pub fn set_day_alarm(&mut self, data: u8, enabled: bool) -> Result<(), E> {
        self.write_register(RegisterTable::CompatibleWeekDayAlarm, Self::to_bcd(data & 0b01111111) | (enabled as u8) << 7)
    }

    pub fn set_timer_counter0(&mut self, data: u8) -> Result<(), E> {
        self.write_register(RegisterTable::CompatibleTimerCounter0, data)
    }

    pub fn set_timer_counter1(&mut self, data: u8) -> Result<(), E> {
        self.write_register(RegisterTable::CompatibleTimerCounter1, data)
    }

    pub fn set_timer_counter(&mut self, data: u16) -> Result<(), E> {
        self.write_register(RegisterTable::CompatibleTimerCounter0, (data & 0x00FF) as u8)?;
        self.write_register(RegisterTable::CompatibleTimerCounter1, ((data & 0xFF00) >> 8) as u8)
    }
}

impl<I2C, E> Rx8900<I2C>
where
    I2C: Read<Error = E> + WriteRead<Error = E> + Write<Error = E>,
{
    pub fn init(&mut self) -> Result<(), E> {
        self.set_te(false)?;
        self.set_fsel0(false)?;
        self.set_fsel1(false)?;
        self.set_test(false)?;
        self.set_vdet()?;
        self.set_vlf()?;
        self.set_aie(false)?;
        self.set_tie(false)?;
        self.set_uie(false)?;

        // set VDETOFF=”1”
        self.set_voltage_detector_off(false)?;
        // set SWOFF=”1”
        self.set_switch_off(true)?;
        Ok(())
    }

    pub fn datetime(&mut self) -> Result<NaiveDateTime, E> {
        let sec = self.sec()?;
        let min = self.min()?;
        let hour = self.hour()?;
        let day = self.day()?;
        let month = self.month()?;
        let yy = self.year()?;
        let date = NaiveDate::from_ymd_opt(2000 + yy as i32, month as u32, day as u32).unwrap();

        let mut time = NaiveTime::from_hms_opt(hour as u32, min as u32, sec as u32);
        if time.is_none() {
            time = NaiveTime::from_hms_opt(0, 0, 0);
        }
        Ok(NaiveDateTime::new(
            date,
            time.unwrap(),
        ))
    }
    pub fn set_datetime(&mut self, data: NaiveDateTime) -> Result<(), E> {
        let date = data.date();
        let yy = date.year() % 100;
        self.set_year(yy as u8)?;
        self.set_month(date.month() as u8)?;
        self.set_day(date.day() as u8)?;
        self.set_week(date.weekday())?;
        let time = data.time();
        self.set_hour(time.hour() as u8)?;
        self.set_min(time.minute() as u8)?;
        self.set_sec(time.second() as u8)?;
        Ok(())
    }

    // pub fn get_time(&mut self) -> Result<NaiveDateTime, E>
    // {
    //     // Implement time retrieval logic here
    //     Ok(NaiveDateTime::parse_from_str("2015-09-05 23:56:04", "%Y-%m-%d %H:%M:%S").unwrap())
    // }

    fn set_bit(&mut self, register: RegisterTable, bit: u8, data: bool) -> Result<(), E> {
        let current = self.read_register(register)?;
        let data = current & !(1 << bit) | (data as u8) << bit;
        self.write_register(register, data)
    }

    pub fn set_alarm_type(&mut self, data: AlarmType) -> Result<(), E> {
        self.set_wada(data == AlarmType::DayAlarm)
    }

    pub fn set_test(&mut self, data: bool) -> Result<(), E> {
        self.set_bit(RegisterTable::CompatibleExtensionRegister, 7, data)
    }

    pub fn set_wada(&mut self, data: bool) -> Result<(), E> {
        self.set_bit(RegisterTable::CompatibleExtensionRegister, 6, data)
    }

    pub fn set_update_interrupt_type(&mut self, data: UpdateInterruptType) -> Result<(), E> {
        self.set_usel(data == UpdateInterruptType::EveryMinute)
    }

    pub fn set_usel(&mut self, data: bool) -> Result<(), E> {
        self.set_bit(RegisterTable::CompatibleExtensionRegister, 5, data)
    }

    pub fn set_te(&mut self, data: bool) -> Result<(), E> {
        self.set_bit(RegisterTable::CompatibleExtensionRegister, 4, data)
    }

    pub fn set_timer_enable(&mut self) -> Result<(), E> {
        self.set_te(true)
    }

    pub fn reset_timer_enable(&mut self) -> Result<(), E> {
        self.set_te(false)
    }

    pub fn set_fsel1(&mut self, data: bool) -> Result<(), E> {
        self.set_bit(RegisterTable::CompatibleExtensionRegister, 3, data)
    }

    pub fn set_fsel0(&mut self, data: bool) -> Result<(), E> {
        self.set_bit(RegisterTable::CompatibleExtensionRegister, 2, data)
    }

    pub fn set_fsel(&mut self, data: u8) -> Result<(), E> {
        let current = self.read_register(RegisterTable::CompatibleExtensionRegister)?;
        let data = current & 0b11110011 | (data as u8) << 2;
        self.write_register(RegisterTable::CompatibleExtensionRegister, data)
    }

    pub fn set_fout_frequency(&mut self, data: FoutFrequency) -> Result<(), E> {
        self.set_fsel(data as u8)
    }

    pub fn set_source_clock(&mut self, data: SourceClock) -> Result<(), E> {
        let current = self.read_register(RegisterTable::CompatibleExtensionRegister)?;
        let data = current & 0b11111100 | (data as u8) << 0;
        self.write_register(RegisterTable::CompatibleExtensionRegister, data)
    }

    pub fn set_tsel1(&mut self, data: bool) -> Result<(), E> {
        self.set_bit(RegisterTable::CompatibleExtensionRegister, 1, data)
    }

    pub fn set_tsel0(&mut self, data: bool) -> Result<(), E> {
        self.set_bit(RegisterTable::CompatibleExtensionRegister, 0, data)
    }

    pub fn set_tsel(&mut self, data: u8) -> Result<(), E> {
        let current = self.read_register(RegisterTable::CompatibleExtensionRegister)?;
        let data = current & 0b11111100 | (data as u8) << 0;
        self.write_register(RegisterTable::CompatibleExtensionRegister, data)
    }

    pub fn set_uf(&mut self) -> Result<(), E> {
        self.set_bit(RegisterTable::CompatibleFlagRegister, 5, false)
    }

    pub fn set_update_flag(&mut self) -> Result<(), E> {
        self.set_uf()
    }

    pub fn set_tf(&mut self, data: bool) -> Result<(), E> {
        self.set_bit(RegisterTable::CompatibleFlagRegister, 4, data)
    }

    pub fn reset_timer_flag(&mut self) -> Result<(), E> {
        self.set_tf(false)
    }

    pub fn set_af(&mut self, data: bool) -> Result<(), E> {
        self.set_bit(RegisterTable::CompatibleFlagRegister, 3, data)
    }

    pub fn reset_alarm_flag(&mut self) -> Result<(), E> {
        self.set_af(false)
    }

    pub fn set_vlf(&mut self) -> Result<(), E> {
        self.set_bit(RegisterTable::CompatibleFlagRegister, 1, false)
    }

    pub fn set_voltage_low_flag(&mut self) -> Result<(), E> {
        self.set_vlf()
    }

    pub fn set_vdet(&mut self) -> Result<(), E> {
        self.set_bit(RegisterTable::CompatibleFlagRegister, 0, false)
    }

    pub fn set_voltage_detect_flag(&mut self) -> Result<(), E> {
        self.set_vdet()
    }

    pub fn set_csel1(&mut self, data: bool) -> Result<(), E> {
        self.set_bit(RegisterTable::CompatibleControlRegister, 7, data)
    }

    pub fn set_csel0(&mut self, data: bool) -> Result<(), E> {
        self.set_bit(RegisterTable::CompatibleControlRegister, 6, data)
    }

    pub fn set_csel(&mut self, data: u8) -> Result<(), E> {
        let current = self.read_register(RegisterTable::CompatibleControlRegister)?;
        let data = (current & 0b00111111) | (data << 6);
        self.write_register(RegisterTable::CompatibleControlRegister, data)
    }

    pub fn set_compensation_interval_type(&mut self, data: CompensationIntervalType) -> Result<(), E> {
        self.set_csel(data as u8)
    }

    pub fn set_uie(&mut self, data: bool) -> Result<(), E> {
        self.set_bit(RegisterTable::CompatibleControlRegister, 5, data)
    }

    pub fn set_update_interrupt_enable(&mut self, data: bool) -> Result<(), E> {
        self.set_uie(data)
    }

    pub fn set_tie(&mut self, data: bool) -> Result<(), E> {
        self.set_bit(RegisterTable::CompatibleControlRegister, 4, data)
    }

    pub fn set_timer_interrupt_enable(&mut self) -> Result<(), E> {
        self.set_tie(true)
    }

    pub fn reset_timer_interrupt_enable(&mut self) -> Result<(), E> {
        self.set_tie(false)
    }

    pub fn set_aie(&mut self, data: bool) -> Result<(), E> {
        self.set_bit(RegisterTable::CompatibleControlRegister, 3, data)
    }

    pub fn set_alarm_interrupt_enable(&mut self) -> Result<(), E> {
        self.set_aie(true)
    }

    pub fn reset_alarm_interrupt_enable(&mut self) -> Result<(), E> {
        self.set_aie(false)
    }

    pub fn set_reset(&mut self, data: bool) -> Result<(), E> {
        self.set_bit(RegisterTable::CompatibleControlRegister, 0, data)
    }

    pub fn set_vdetoff(&mut self, data: bool) -> Result<(), E> {
        self.set_bit(RegisterTable::ExtendedBackupFunction, 3, data)
    }

    pub fn set_voltage_detector_off(&mut self, data: bool) -> Result<(), E> {
        self.set_vdetoff(data)
    }

    pub fn set_swoff(&mut self, data: bool) -> Result<(), E> {
        self.set_bit(RegisterTable::ExtendedBackupFunction, 2, data)
    }

    pub fn set_switch_off(&mut self, data: bool) -> Result<(), E> {
        self.set_swoff(data)
    }

    pub fn set_bksmp1(&mut self, data: bool) -> Result<(), E> {
        self.set_bit(RegisterTable::ExtendedBackupFunction, 1, data)
    }

    pub fn set_bksmp0(&mut self, data: bool) -> Result<(), E> {
        self.set_bit(RegisterTable::ExtendedBackupFunction, 0, data)
    }

    pub fn set_bksmp(&mut self, data: u8) -> Result<(), E> {
        let current = self.read_register(RegisterTable::ExtendedBackupFunction)?;
        let data = (current & 0b11111100) | (data & 0b00000011);
        self.write_register(RegisterTable::ExtendedBackupFunction, data)
    }
    

    pub fn set_backup_mode_sampling_time(&mut self, data: u8) -> Result<(), E> {
        self.set_bksmp(data)
    }
}
