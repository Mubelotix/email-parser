use email_parser::prelude::*;
use std::fs::*;

#[ignore]
#[test]
fn enron() {
    let mut ok = 0;
    let mut tot = 0;
    'main: for dir in read_dir("/home/mubelotix/Downloads/enron_mail_20150507/maildir").unwrap() {
        if let Ok(dir) = dir {
            if let Ok(entries) = read_dir(dir.path()) {
                for dir in entries {
                    if let Ok(dir) = dir {
                        if let Ok(entries) = read_dir(dir.path()) {
                            for dir in entries {
                                if let Ok(file) = dir {
                                    if file.file_type().map(|f| f.is_file()).ok() == Some(true) {
                                        if let Ok(content) = read(file.path()) {
                                            let result = Email::parse(&content);

                                            tot += 1;
                                            match result {
                                                Ok(_email) => ok += 1,
                                                Err(e) => println!("{} at {:?}", e, file.path()),
                                            }

                                            if tot >= 500000 {
                                                break 'main;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    println!("{}/{}", ok, tot)
}

#[test]
fn test_date_timezone_abbr() {
    // Test timezone abbreviations
    assert_success("Date", "Tue, 01 Nov 2016 00:19:05 GMT");
}

#[test]
fn test_date_singular_hour() {
    // Test singular hours
    assert_success("Date", "Thu, 3 Dec 2020 9:20:16 +0100");
}

#[test]
fn test_subject_latin1() {
    #[cfg(feature = "mime")]
    {
        // Test Latin-1 chars in subject
        assert_success("Subject", "Neuer, privater Eintrag fьr dich");
    }
}

#[test]
fn test_display_name_dot() {
    #[cfg(feature = "mime")]
    {
        // Test '.' in Subject display name
        assert_success("From", "Example.com <webmaster@example.com>");
    }
}

#[test]
fn test_display_name_at() {
    #[cfg(feature = "mime")]
    {
        // Test '@' in Subject display name
        assert_success("From", "bb.b@exxx.de <bb.b@exxx.de>");
    }
}

#[test]
fn test_date_two_digits_year() {
    // Test support for a year with only two digits
    assert_success("Date", "Wed, 15 Sep 10 13:53:08 +0200");
}

#[test]
fn test_display_name_unicode() {
    // Test support unicode in display name
    assert_success("From", "\"Hey M端n look!\" <noreply@example.com>");
}

#[test]
fn test_timezone_colon() {
    // Test support for a timezone with the '+00:00' format
    assert_success("Date", "Fri, 04 Aug 2006 11:52:00 +00:00");
}

#[test]
fn test_timezone_meta() {
    // Test support for a timezone with the additional meta info at the end
    assert_success(
        "Date",
        "Thu, 31 Aug 2006 22:03:40 +0200 (Westeuropäische Sommerzeit)",
    );
}

#[test]
fn test_angle_right_whitespace() {
    // Test support for additional whitespace after the angle bracket at the end
    assert_success("From", "\"Ryan Riddle\" <hello@forrst.com> ");
}

#[test]
fn test_random_unicode() {
    // If you check a hex editor, you'll find unicode chars in the Feedback-ID line at the end
    assert_success("Feedback-ID", "Feedback-ID: 15bbe728-699d-4fab-9ed5-2551c7f2fd70:b4cad6e3-63d4-4776-bd51-33e0d8fe639a:email:epslh1 ");
}

#[test]
fn test_normal_email_works() {
    assert_success("To", "author@gmail.com");
}

/// Generate a fake email with valid information except for the
/// invalid information added via the override flags
fn assert_success(override_key: &str, override_value: &str) {
    let keys = &[
        ("Delivered-To", "example@example.com"),
        ("To", "example@example.com"),
        ("From", "from@example.com"),
        ("Sender", "sender@example.com"),
        ("Date", "Thu, 3 Dec 2020 9:20:16 +0100"),
        ("Content-Type", "text/plain; charset=\"utf-8\""),
        (
            "Message-ID",
            "<aaacddda-4848-8888-9999-39652ca9c15c@aaa2a01c9u286.aa.local>",
        ),
        ("Subject", "Hello World"),
        ("Content-Transfer-Encoding", "7bit"),
        ("Feedback-ID", "klasjdfkladsfjkladsjfkl"),
    ];
    let headers: Vec<String> = keys
        .iter()
        .map(|(key, value)| {
            if key == &override_key {
                format!("{}: {}", override_key, override_value)
            } else {
                format!("{}: {}", key, value)
            }
        })
        .collect();
    let joined = headers.join("\r\n");
    let message = format!("{}\r\n\r\nBody", &joined);
    let email = Email::parse(&message.as_bytes());
    assert_eq!(email.err(), None)
}

#[cfg(feature = "mime")]
fn my_emails() {
    let mut ok = 0;
    let mut tot = 0;
    'main: for file in read_dir("/home/mubelotix/Downloads/emails").unwrap() {
        if let Ok(file) = file {
            if file.file_type().map(|f| f.is_file()).ok() == Some(true) {
                if let Ok(mut content) = read(file.path()) {
                    let mut i = 0;
                    loop {
                        match content.get(i) {
                            None => break,
                            Some(b'\n') => match content.get(i.saturating_sub(1)) {
                                Some(b'\r') => (),
                                Some(_) => content.insert(i, b'\r'),
                                None => (),
                            },
                            _ => (),
                        }
                        i += 1;
                    }

                    let result = Email::parse(&content);

                    tot += 1;
                    match result {
                        Ok(email) => {
                            println!("{:?}", email.subject);
                            ok += 1
                        }
                        Err(e) => println!("{} at {:?}", e, file.path()),
                    }

                    if tot >= 500000 {
                        break 'main;
                    }
                }
            }
        }
    }
    assert_eq!(ok, 332);
    println!("{}/{}", ok, tot)
}
