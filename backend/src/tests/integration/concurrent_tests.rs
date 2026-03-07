use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::tests::utils::{random_bucket_prefix, temp_sqlite_path, unique_email};

#[actix_rt::test]
async fn test_concurrent_bucket_prefix_generation() {
    let counter = Arc::new(AtomicUsize::new(0));
    let mut handles = vec![];

    for _ in 0..10 {
        let counter_clone = Arc::clone(&counter);
        let handle = tokio::spawn(async move {
            let prefix = random_bucket_prefix();
            counter_clone.fetch_add(1, Ordering::SeqCst);
            prefix
        });
        handles.push(handle);
    }

    let prefixes: Vec<String> = futures::future::join_all(handles)
        .await
        .into_iter()
        .map(|h| h.unwrap())
        .collect();

    assert_eq!(prefixes.len(), 10, "Should have 10 prefixes");
    assert_eq!(
        prefixes.len(),
        prefixes.iter().collect::<std::collections::HashSet<_>>().len(),
        "All prefixes should be unique"
    );
    assert_eq!(counter.load(Ordering::SeqCst), 10, "Counter should be 10");
}

#[actix_rt::test]
async fn test_concurrent_email_generation() {
    let counter = Arc::new(AtomicUsize::new(0));
    let mut handles = vec![];

    for _ in 0..10 {
        let counter_clone = Arc::clone(&counter);
        let handle = tokio::spawn(async move {
            let email = unique_email();
            counter_clone.fetch_add(1, Ordering::SeqCst);
            email
        });
        handles.push(handle);
    }

    let emails: Vec<String> = futures::future::join_all(handles)
        .await
        .into_iter()
        .map(|h| h.unwrap())
        .collect();

    assert_eq!(emails.len(), 10, "Should have 10 emails");
    assert_eq!(
        emails.len(),
        emails.iter().collect::<std::collections::HashSet<_>>().len(),
        "All emails should be unique"
    );
    assert_eq!(counter.load(Ordering::SeqCst), 10, "Counter should be 10");
}

#[actix_rt::test]
async fn test_concurrent_path_generation() {
    let counter = Arc::new(AtomicUsize::new(0));
    let mut handles = vec![];

    for _ in 0..10 {
        let counter_clone = Arc::clone(&counter);
        let handle = tokio::spawn(async move {
            let path = temp_sqlite_path();
            counter_clone.fetch_add(1, Ordering::SeqCst);
            path
        });
        handles.push(handle);
    }

    let paths: Vec<String> = futures::future::join_all(handles)
        .await
        .into_iter()
        .map(|h| h.unwrap())
        .collect();

    assert_eq!(paths.len(), 10, "Should have 10 paths");
    assert_eq!(
        paths.len(),
        paths.iter().collect::<std::collections::HashSet<_>>().len(),
        "All paths should be unique"
    );
    assert_eq!(counter.load(Ordering::SeqCst), 10, "Counter should be 10");
}

#[actix_rt::test]
async fn test_concurrent_isolation_resources() {
    let mut handles = vec![];

    for i in 0..5 {
        let handle = tokio::spawn(async move {
            let bucket_prefix = random_bucket_prefix();
            let temp_path = temp_sqlite_path();
            let email = unique_email();

            (bucket_prefix, temp_path, email, i)
        });
        handles.push(handle);
    }

    let results: Vec<(String, String, String, usize)> = futures::future::join_all(handles)
        .await
        .into_iter()
        .map(|h| h.unwrap())
        .collect();

    let prefixes: Vec<String> = results.iter().map(|r| r.0.clone()).collect();
    let paths: Vec<String> = results.iter().map(|r| r.1.clone()).collect();
    let emails: Vec<String> = results.iter().map(|r| r.2.clone()).collect();

    assert_eq!(
        prefixes.len(),
        prefixes.iter().collect::<std::collections::HashSet<_>>().len(),
        "All bucket prefixes should be unique"
    );
    assert_eq!(
        paths.len(),
        paths.iter().collect::<std::collections::HashSet<_>>().len(),
        "All temp paths should be unique"
    );
    assert_eq!(
        emails.len(),
        emails.iter().collect::<std::collections::HashSet<_>>().len(),
        "All emails should be unique"
    );
}

#[actix_rt::test]
#[ignore = "Requires MailHog service running"]
async fn test_concurrent_mailhog_emails() {
    let test_emails: Vec<String> = (0..3).map(|_| unique_email()).collect();

    let mut send_handles = vec![];
    for (i, email) in test_emails.iter().enumerate() {
        let subject = format!("Concurrent Email {}", i);
        let body = format!("Body for concurrent email {}", i);
        let email_clone = email.clone();

        let handle = tokio::spawn(async move {
            let result = crate::tests::integration::mailhog_tests::send_email(
                &email_clone,
                &subject,
                &body,
            )
            .await;
            (email_clone, result)
        });
        send_handles.push(handle);
    }

    let send_results = futures::future::join_all(send_handles).await;
    for (email, result) in send_results.into_iter().map(|h| h.unwrap()) {
        assert!(result.is_ok(), "Should send email to {}", email);
    }

    tokio::time::sleep(std::time::Duration::from_millis(1000)).await;

    for email in test_emails {
        let messages = crate::tests::integration::mailhog_tests::get_mailhog_messages(&email)
            .await
            .unwrap();
        assert!(
            !messages.is_empty(),
            "Should have messages for {}",
            email
        );
    }
}