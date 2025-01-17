/*
 * Hurl (https://hurl.dev)
 * Copyright (C) 2024 Orange
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *          http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 */
use hurl_core::ast::{Pos, SourceInfo};

use crate::output::Error;
use crate::runner;
use crate::runner::{HurlResult, Output};
use crate::util::term::Stdout;

/// Writes the `hurl_result` last response to the file `filename_out`.
///
/// If `filename_out` is `None`, standard output is used. If `include_headers` is true, the last
/// HTTP response headers are written before the body response.
pub fn write_last_body(
    hurl_result: &HurlResult,
    include_headers: bool,
    color: bool,
    filename_out: Option<&Output>,
    stdout: &mut Stdout,
) -> Result<(), Error> {
    // Get the last call of the Hurl result.
    let Some(last_entry) = &hurl_result.entries.last() else {
        return Ok(());
    };
    let Some(call) = &last_entry.calls.last() else {
        return Ok(());
    };
    let response = &call.response;
    let mut output = vec![];

    // If include options is set, we output the HTTP response headers
    // with status and version (to mimic curl outputs)
    if include_headers {
        let mut text = response.get_status_line_headers(color);
        text.push('\n');
        output.append(&mut text.into_bytes());
    }
    if last_entry.compressed {
        let mut bytes = match response.uncompress_body() {
            Ok(b) => b,
            Err(e) => {
                // FIXME: we convert to a runner::Error to be able to use fixme!
                // We may pass a [`SourceInfo`] as a parameter of this method to make
                // a more accurate error
                let source_info = SourceInfo::new(Pos::new(0, 0), Pos::new(0, 0));
                let error = runner::Error::new(source_info, e.into(), false);
                return Err(error.into());
            }
        };
        output.append(&mut bytes);
    } else {
        let bytes = &response.body;
        output.extend(bytes);
    }
    match filename_out {
        Some(out) => out.write(&output, stdout, None)?,
        None => Output::Stdout.write(&output, stdout, None)?,
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::http::{Call, Header, HeaderVec, HttpVersion, Request, Response};
    use crate::output::write_last_body;
    use crate::runner::{EntryResult, HurlResult, Output};
    use crate::util::term::{Stdout, WriteMode};
    use hurl_core::ast::{Pos, SourceInfo};

    fn hurl_result() -> HurlResult {
        let mut headers = HeaderVec::new();
        headers.push(Header::new("x-foo", "xxx"));
        headers.push(Header::new("x-bar", "yyy0"));
        headers.push(Header::new("x-bar", "yyy1"));
        headers.push(Header::new("x-bar", "yyy2"));
        headers.push(Header::new("x-baz", "zzz"));

        HurlResult {
            entries: vec![
                EntryResult {
                    entry_index: 1,
                    source_info: SourceInfo::new(Pos::new(0, 0), Pos::new(0, 0)),
                    calls: vec![Call {
                        request: Request {
                            url: "https://foo.com".to_string(),
                            method: "GET".to_string(),
                            headers: HeaderVec::new(),
                            body: vec![],
                        },
                        response: Default::default(),
                        timings: Default::default(),
                    }],
                    captures: vec![],
                    asserts: vec![],
                    errors: vec![],
                    time_in_ms: 0,
                    compressed: false,
                },
                EntryResult {
                    entry_index: 2,
                    source_info: SourceInfo::new(Pos::new(0, 0), Pos::new(0, 0)),
                    calls: vec![Call {
                        request: Request {
                            url: "https://bar.com".to_string(),
                            method: "GET".to_string(),
                            headers: HeaderVec::new(),
                            body: vec![],
                        },
                        response: Default::default(),
                        timings: Default::default(),
                    }],
                    captures: vec![],
                    asserts: vec![],
                    errors: vec![],
                    time_in_ms: 0,
                    compressed: false,
                },
                EntryResult {
                    entry_index: 3,
                    source_info: SourceInfo::new(Pos::new(0, 0), Pos::new(0, 0)),
                    calls: vec![Call {
                        request: Request {
                            url: "https://baz.com".to_string(),
                            method: "GET".to_string(),
                            headers: HeaderVec::new(),
                            body: vec![],
                        },
                        response: Response {
                            version: HttpVersion::Http3,
                            status: 204,
                            headers,
                            body: b"{\"say\": \"Hello World!\"}".into(),
                            duration: Default::default(),
                            url: "".to_string(),
                            certificate: None,
                        },
                        timings: Default::default(),
                    }],
                    captures: vec![],
                    asserts: vec![],
                    errors: vec![],
                    time_in_ms: 0,
                    compressed: false,
                },
            ],
            time_in_ms: 100,
            success: true,
            cookies: vec![],
            timestamp: 0,
        }
    }

    #[test]
    fn write_last_body_with_headers() {
        let result = hurl_result();
        let include_header = true;
        let color = false;
        let output = Some(Output::Stdout);
        let mut stdout = Stdout::new(WriteMode::Buffered);

        write_last_body(&result, include_header, color, output.as_ref(), &mut stdout).unwrap();
        let stdout = String::from_utf8(stdout.buffer().to_vec()).unwrap();
        assert_eq!(
            stdout,
            "HTTP/3 204\n\
             x-foo: xxx\n\
             x-bar: yyy0\n\
             x-bar: yyy1\n\
             x-bar: yyy2\n\
             x-baz: zzz\n\
             \n\
             {\"say\": \"Hello World!\"}"
        );
    }
}
