use actix_multipart_test::MultiPartFormDataBuilder;
use std::path::{Path, PathBuf};

const TEST_DATA_PATH: &str = "tests/test_data";
pub const VIDEO1: TestFile = TestFile {
    dir_path: TEST_DATA_PATH,
    file_name: "video1.mp4",
    test_file_type: TestFileType::Video,
};

pub const VIDEO2: TestFile = TestFile {
    dir_path: TEST_DATA_PATH,
    file_name: "video2.mp4",
    test_file_type: TestFileType::Video,
};

pub const PLACEHOLDER_PNG: TestFile = TestFile {
    dir_path: TEST_DATA_PATH,
    file_name: "placeholder.png",
    test_file_type: TestFileType::Image,
};

pub struct TestFile {
    dir_path: &'static str,
    file_name: &'static str,
    test_file_type: TestFileType,
}

impl TestFile {
    pub fn get_path_to_file(&self) -> PathBuf {
        Path::new(&self.dir_path)
            .join(self.test_file_type.get_folder_name())
            .join(self.file_name)
    }

    pub fn get_multipart_builder(
        &self,
        property_name: impl Into<String>,
        content_type: impl Into<String>,
    ) -> ((String, String), Vec<u8>) {
        let mut builder = MultiPartFormDataBuilder::new();

        builder
            .with_file(
                self.get_path_to_file(),
                property_name,
                content_type,
                self.file_name,
            )
            .build()
    }
}

#[allow(dead_code)]
enum TestFileType {
    Video,
    Image,
    Other,
}

impl TestFileType {
    pub fn get_folder_name(&self) -> &str {
        match self {
            TestFileType::Video => "videos",
            TestFileType::Image => "images",
            TestFileType::Other => "other",
        }
    }
}
