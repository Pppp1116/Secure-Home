use esp_hal::lcd_cam::cam::Camera as esp_cam;

pub struct Camera {
    camera: esp_cam<'static>,
}
impl Camera {
    pub(crate) fn new(camera: esp_cam<'static>) -> Self {
        Self { camera }
    }
}
