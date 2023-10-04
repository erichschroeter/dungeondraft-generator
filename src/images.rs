use std::path::{Path, PathBuf};

use log::{debug, info};
use opencv::prelude::*;
use opencv::imgcodecs::{imread, imwrite};
use opencv::imgproc;
use opencv::core::{self, Scalar};
use opencv::types::VectorOfMat;

#[derive(Debug)]
pub struct Point {
    x: i32,
    y: i32,
}

impl std::fmt::Display for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

#[derive(Debug)]
pub struct Shape {
    vertice_count: u32,
    coordinates: Point,
    contour: Mat,
}

impl std::fmt::Display for Shape {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} vertices @ {} : {:?}", self.vertice_count, self.coordinates, self.contour)
    }
}

pub fn find_shapes(image_path: &Path) -> Result<Vec<Shape>, Box<dyn std::error::Error>> {
    debug!("Analyzing {}", image_path.display());
    // let image = Reader::open(o).unwrap().decode().unwrap().to_rgb8();
    let image = imread(image_path.as_os_str().to_str().unwrap(), opencv::imgcodecs::ImreadModes::IMREAD_COLOR as i32)?;

    // Convert the image to grayscale
    let mut gray_image = Mat::default();
    imgproc::cvt_color(&image, &mut gray_image, imgproc::COLOR_BGR2GRAY, 0)?;

    // Apply edge detection (e.g. using the Canny algorithm)
    let mut edges = Mat::default();
    imgproc::canny(&gray_image, &mut edges, 50.0, 150.0, 3, false)?;

    // Find contours in the edge-detected image
    let mut contours = VectorOfMat::new();
    let mut hierarchy = Mat::default();
    imgproc::find_contours_with_hierarchy(
        &mut edges,
        &mut contours,
        &mut hierarchy,
        imgproc::RETR_EXTERNAL,
        imgproc::CHAIN_APPROX_SIMPLE,
        core::Point::new(0, 0),
    )?;

    // Create a new image to draw contours on
    let mut image_with_contours = Mat::default();
    image.copy_to(&mut image_with_contours)?;

    // Iterate over detected contours and print their coords and dimensions
    info!("Detected {} contours", contours.len());
    let mut shapes = Vec::new();
    for contour in contours.iter() {
        let area = imgproc::contour_area(&contour, false)?;
        if area > 100.0 {
            let mut approx = Mat::default();
            let epsilon = 0.04 * imgproc::arc_length(&contour, true)?;
            imgproc::approx_poly_dp(&contour, &mut approx, epsilon, true)?;
            let num_vertices = approx.total() as u32;
            let bounding_rect = imgproc::bounding_rect(&contour)?;
            let shape = Shape {
                vertice_count: num_vertices,
                coordinates: Point {
                    x: bounding_rect.x,
                    y: bounding_rect.y,
                },
                contour,
            };
            info!("{}", shape);
            shapes.push(shape);
        }
    }
    info!("{:?}", shapes);
    Ok(shapes)
}

pub fn trace_shapes(image_path: &Path) -> Result<PathBuf, Box<dyn std::error::Error>> {
    debug!("Analyzing {}", image_path.display());
    // let image = Reader::open(o).unwrap().decode().unwrap().to_rgb8();
    let image = imread(image_path.as_os_str().to_str().unwrap(), opencv::imgcodecs::ImreadModes::IMREAD_COLOR as i32)?;

    // Convert the image to grayscale
    let mut gray_image = Mat::default();
    imgproc::cvt_color(&image, &mut gray_image, imgproc::COLOR_BGR2GRAY, 0)?;

    // Apply edge detection (e.g. using the Canny algorithm)
    let mut edges = Mat::default();
    imgproc::canny(&gray_image, &mut edges, 50.0, 150.0, 3, false)?;

    // Find contours in the edge-detected image
    let mut contours = VectorOfMat::new();
    let mut hierarchy = Mat::default();
    imgproc::find_contours_with_hierarchy(
        &mut edges,
        &mut contours,
        &mut hierarchy,
        imgproc::RETR_EXTERNAL,
        imgproc::CHAIN_APPROX_SIMPLE,
        core::Point::new(0, 0),
    )?;

    // Create a new image to draw contours on
    let mut image_with_contours = Mat::default();
    image.copy_to(&mut image_with_contours)?;

    // Iterate over detected contours and print their coords and dimensions
    info!("Detected {} contours", contours.len());
    let mut contour_count = 0;
    for contour in contours.iter() {
        let area = imgproc::contour_area(&contour, false)?;
        if area > 100.0 {
            let mut approx = Mat::default();
            let epsilon = 0.04 * imgproc::arc_length(&contour, true)?;
            imgproc::approx_poly_dp(&contour, &mut approx, epsilon, true)?;
            let num_vertices = approx.total() as i32;
            if num_vertices == 6 {
                info!("Detected hexagon");
            } else if num_vertices == 5 {
                info!("Detected pentagon");
            } else if num_vertices == 4 {
                info!("Detected quadrilateral");
            } else if num_vertices == 3 {
                info!("Detected triangle");
            } else {
                info!("Detected polygon with {} vertices", num_vertices);
            }
            // let points = approx.iter::<core::Point2f>()?;
            // for point in points {
            //     let x = point.0 as i32;
            //     let y = point.1 as i32;
            //     warn!("Shape coord");
            // }

            let bounding_rect = imgproc::bounding_rect(&contour)?;
            contour_count = contour_count + 1;
            debug!(
                "[{} / {}] Shape detected at ({}, {}) with width: {} and height {}",
                contour_count,
                contours.len(),
                bounding_rect.x,
                bounding_rect.y,
                bounding_rect.width,
                bounding_rect.height,
            );

            // Draw contours on the image
            let color = Scalar::new(0.0, 255.0, 0.0, 0.0);
            imgproc::draw_contours(
                &mut image_with_contours,
                &contours,
                -1,
                color,
                2,
                opencv::core::LINE_8,
                &hierarchy,
                1,
                core::Point::new(0, 0),
            )?;
        }
    }

    let mut contour_image_path = image_path.to_path_buf();
    contour_image_path.set_extension("shapes.png");
    debug!("Generating shapes image {}", contour_image_path.display());
    // Save the iamge with contours
    imwrite(contour_image_path.as_os_str().to_str().unwrap(), &image_with_contours, &core::Vector::new())?;
    Ok(contour_image_path)
}
