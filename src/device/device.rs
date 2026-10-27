use crate::device::camera::Camera as MyCam;
use core::default::Default;
use esp_hal::lcd_cam::cam::Camera;
use esp_hal::peripherals::*;
use esp_hal::time::Rate;
use esp_hal::{
    Config,
    clock::CpuClock,
    peripherals::{FROM_CPU_INTR0, GPIO14, TIMG0},
    timer::timg::TimerGroup,
};
use panic_rtt_target as _;

use crate::device::pir::Pir;

#[derive(Debug)]
pub enum EspError {
    Camera(esp_hal::lcd_cam::cam::ConfigError),
}
impl From<esp_hal::lcd_cam::cam::ConfigError> for EspError {
    fn from(value: esp_hal::lcd_cam::cam::ConfigError) -> Self {
        EspError::Camera(value)
    }
}
pub type Result<T> = core::result::Result<T, EspError>;

pub struct UnconfiguredCamera {
    mclk_pin: GPIO15<'static>,
    vsync_pin: GPIO6<'static>,
    href_pin: GPIO7<'static>,
    pclk_pin: GPIO13<'static>,
    lcd_cam: LCD_CAM<'static>,
    dma_ch0: DMA_CH0<'static>,

    data0_pin: GPIO11<'static>,
    data1_pin: GPIO9<'static>,
    data2_pin: GPIO8<'static>,
    data3_pin: GPIO10<'static>,
    data4_pin: GPIO12<'static>,
    data5_pin: GPIO18<'static>,
    data6_pin: GPIO17<'static>,
    data7_pin: GPIO16<'static>,
}
pub struct UnconfiguredPir(GPIO14<'static>);

pub struct Device<CameraState = UnconfiguredCamera, PIRState = UnconfiguredPir> {
    config: Config,
    camera: CameraState,
    pir: PIRState,
}

impl Default for Device<UnconfiguredCamera, UnconfiguredPir> {
    fn default() -> Self {
        let config = Config::default().with_cpu_clock(CpuClock::max());
        let peripherals = esp_hal::init(config);
        let pir = UnconfiguredPir(peripherals.GPIO14);

        start_hardware_timer(peripherals.TIMG0, peripherals.FROM_CPU_INTR0);

        let camera = UnconfiguredCamera {
            mclk_pin: peripherals.GPIO15,
            vsync_pin: peripherals.GPIO6,
            href_pin: peripherals.GPIO7,
            pclk_pin: peripherals.GPIO13,
            lcd_cam: peripherals.LCD_CAM,
            dma_ch0: peripherals.DMA_CH0,

            data0_pin: peripherals.GPIO11,
            data1_pin: peripherals.GPIO9,
            data2_pin: peripherals.GPIO8,
            data3_pin: peripherals.GPIO10,
            data4_pin: peripherals.GPIO12,
            data5_pin: peripherals.GPIO18,
            data6_pin: peripherals.GPIO17,
            data7_pin: peripherals.GPIO16,
        };
        Self {
            config,
            pir,
            camera: camera,
        }
    }
}

impl<CameraState> Device<CameraState, UnconfiguredPir> {
    pub fn with_pir(self) -> Device<CameraState, Pir> {
        let sensor = Pir::new(self.pir.0);

        Device {
            config: self.config,
            camera: self.camera,
            pir: sensor,
        }
    }
}

impl<PIR> Device<UnconfiguredCamera, PIR> {
    pub fn with_camera(self) -> Result<Device<MyCam, PIR>> {
        let config = esp_hal::lcd_cam::cam::Config::default().with_frequency(Rate::from_mhz(20));
        let cam = esp_hal::lcd_cam::LcdCam::new(self.camera.lcd_cam).cam;

        let camera = Camera::new(cam, self.camera.dma_ch0, config)?
            .with_master_clock(self.camera.mclk_pin)
            .with_vsync(self.camera.vsync_pin)
            .with_h_enable(self.camera.href_pin)
            .with_pixel_clock(self.camera.pclk_pin)
            .with_data0(self.camera.data0_pin)
            .with_data1(self.camera.data1_pin)
            .with_data2(self.camera.data2_pin)
            .with_data3(self.camera.data3_pin)
            .with_data4(self.camera.data4_pin)
            .with_data5(self.camera.data5_pin)
            .with_data6(self.camera.data6_pin)
            .with_data7(self.camera.data7_pin);

        let camera = MyCam::new(camera);
        Ok(Device::<MyCam, PIR> {
            camera: camera,
            config: self.config,
            pir: self.pir,
        })
    }
}
fn start_hardware_timer(timer: TIMG0<'static>, cpu_intro: FROM_CPU_INTR0<'static>) {
    let timg0 = TimerGroup::new(timer);
    esp_rtos::start(timg0.timer0, cpu_intro);
}
