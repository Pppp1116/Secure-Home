use esp_hal::{
    gpio::{Input, InputConfig, Pull},
    peripherals::{GPIO14, Peripherals},
};

pub struct Pir {
    sensor: Input<'static>,
}

impl Pir {
    pub fn new(pin: GPIO14<'static>) -> Self {
        let input = Input::new(pin, InputConfig::default().with_pull(Pull::None));

        Self { sensor: input }
    }
}
