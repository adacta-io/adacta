use chrono::{DateTime, NaiveDateTime, Utc};
use spectral::assert_that;

use crate::juicer::Juicer as _;
use crate::meta::Metadata;
use crate::repository::Repository;

use super::*;
use log::LevelFilter;

#[tokio::test]
async fn test_extract_pages() {
    let _ = env_logger::builder().filter_module("adacta", LevelFilter::Trace).is_test(true).try_init();

    let repository = Repository::with_path(tempfile::tempdir().unwrap()).await.unwrap();
    let juicer = juicer().await.unwrap();

    let bundle = upload(&repository, Metadata {
        uploaded: DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(1_000_000_000, 0), Utc),
        ..Metadata::new()
    }, "extract_pages.pdf").await.unwrap();

    juicer.extract(&bundle).await.unwrap();

    assert_that!(bundle.read_metadata().await.unwrap().pages).is_equal_to(1);
}

#[tokio::test]
async fn test_extract_with_title() {
    let _ = env_logger::builder().filter_module("adacta", LevelFilter::Trace).is_test(true).try_init();

    let repository = Repository::with_path(tempfile::tempdir().unwrap()).await.unwrap();
    let juicer = juicer().await.unwrap();

    let bundle = upload(&repository, Metadata {
        uploaded: DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(1_000_000_000, 0), Utc),
        ..Metadata::new()
    }, "extract_with_title.pdf").await.unwrap();

    juicer.extract(&bundle).await.unwrap();

    assert_that!(bundle.read_metadata().await.unwrap().title).is_equal_to(Some(String::from("Having a title")));
}

#[tokio::test]
async fn test_extract_without_title() {
    let _ = env_logger::builder().filter_module("adacta", LevelFilter::Trace).is_test(true).try_init();

    let repository = Repository::with_path(tempfile::tempdir().unwrap()).await.unwrap();
    let juicer = juicer().await.unwrap();

    let bundle = upload(&repository, Metadata {
        uploaded: DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(1_000_000_000, 0), Utc),
        title: Some(String::from("Some existing title")),
        ..Metadata::new()
    }, "extract_without_title.pdf").await.unwrap();

    juicer.extract(&bundle).await.unwrap();

    assert_that!(bundle.read_metadata().await.unwrap().title).is_equal_to(Some(String::from("Some existing title")));
}

#[tokio::test]
async fn test_extract_with_title_conflicting() {
    let _ = env_logger::builder().filter_module("adacta", LevelFilter::Trace).is_test(true).try_init();

    let repository = Repository::with_path(tempfile::tempdir().unwrap()).await.unwrap();
    let juicer = juicer().await.unwrap();

    let bundle = upload(&repository, Metadata {
        uploaded: DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(1_000_000_000, 0), Utc),
        title: Some(String::from("Some existing title")),
        ..Metadata::new()
    }, "extract_with_title.pdf").await.unwrap();

    juicer.extract(&bundle).await.unwrap();

    assert_that!(bundle.read_metadata().await.unwrap().title).is_equal_to(Some(String::from("Some existing title")));
}