#[macro_use]
extern crate rocket;

use opencv::{
    core::{Size, Vector},
    imgcodecs::{self, imdecode, imencode, imread},
    imgproc::gaussian_blur,
};
use rocket::{form::Form, fs::TempFile};
#[derive(Debug, FromForm)]
pub struct Blur<'r> {
    pub image: TempFile<'r>,
    pub ksize_width: i32,
    pub ksize_height: i32,
    pub sigma_x: f64,
    // This is according to https://docs.rs/opencv/latest/opencv/imgproc/fn.gaussian_blur.html#c-default-parameters
    #[field(default = 0_f64)]
    pub sigma_y: f64,
    #[field(default = ".png")]
    pub format: &'r str,
}

/// Gets the image as base64 encoded string and returns the blurred image as the same
///
/// In this case everything is done in memory so there is no writes to disk which might
/// significantly slow down the process due to constant reads and writes to the disk
#[post("/blur", format = "application/x-www-form-urlencoded", data = "<data>")]
async fn blur(data: String) -> String {
    // println!("{}", data);
    let image_bytes = base64::decode(data).expect("Failed to decode the base64 encoded data");
    let image_decoded = imdecode(
        &Vector::<u8>::from_iter(image_bytes),
        imgcodecs::IMREAD_COLOR,
    )
    .expect("Failed to decode the image");
    let mut blurred_image = opencv::core::Mat::default();

    gaussian_blur(
        &image_decoded,
        &mut blurred_image,
        Size::new(45, 45),
        0.0,
        0.0,
        4, // same as BORDER_DEFAULT
    )
    .expect("Failed to blur image");

    let mut image = Vector::with_capacity(50 * (2 ^ 23)); // take a buffer of 50 mb

    imencode(".png", &blurred_image, &mut image, &Vector::default())
        .expect("Failed to encode image");
    base64::encode(image)
}

/// Gets the image as a raw file and returns the blurred image as a file as well
///
/// In this case the file is written to disk* before reading so it might be slower than the other
/// one.
///
/// *
/// In case of linux systems the std::env::temp_dir() is /tmp which is usually a ramdisk and I'm
/// writing the file there so there **should** be no slowdowns but I don't know for sure in
/// Windows/Mac
#[post("/blur", format = "multipart/form-data", data = "<form>")]
async fn blur_form(mut form: Form<Blur<'_>>) -> Vec<u8> {
    form.image
        .persist_to(std::env::temp_dir().join("term_file"))
        .await
        .expect("Failed to persist image");

    let image_decoded = imread(
        &form
            .image
            .path()
            .expect("TempFile didn't persist")
            .to_string_lossy(),
        imgcodecs::ImreadModes::IMREAD_UNCHANGED as i32,
    )
    .expect("Failed to decode the image");
    let mut blurred_image = opencv::core::Mat::default();

    gaussian_blur(
        &image_decoded,
        &mut blurred_image,
        Size::new(form.ksize_width, form.ksize_height),
        form.sigma_x,
        form.sigma_y,
        opencv::core::BORDER_DEFAULT,
    )
    .expect("Unable to blur image");

    let mut image = Vector::with_capacity(50 * (2 ^ 23));

    imencode(form.format, &blurred_image, &mut image, &Vector::default())
        .expect("Failed to encode image");
    image.to_vec()
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![blur, blur_form])
}
