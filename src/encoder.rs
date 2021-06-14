use deflate::{deflate_bytes, deflate_bytes_gzip};

use std::io::Cursor;

use rocket::{Request, Response};
use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::Header;
use rocket::log;

pub struct DflateEncoder;

#[derive(Debug, PartialEq, Eq)]
enum ENCODING {
    DEFLATE,
    GZIP,
}

#[rocket::async_trait]
impl Fairing for DflateEncoder {
    fn info(&self) -> Info {
        Info {
            name: "Response compression using dflate algoritm",
            kind: Kind::Response
        }
    }

    async fn on_response<'r>(&self, request: &'r Request<'_>, response: &mut Response<'r>) {

        if response.body().is_none() {
            log::info_!("Encoder: body empty");
            return
        }

        //TODO is some compress algoritms in "content-encoding"
        if response.headers().get_one("content-encoding").is_some() {
            return
        }

        //TODO Chainge vac to hashset
        let accept_encoding_algoritms = request.headers()
            .get("Accept-Encoding")
            .fold(Vec::<&str>::new(), |mut vec, alg| {
                vec.append( &mut alg.split(',').map(|text| text.trim()).collect::<Vec<_>>());
                vec
            });

        let encoding_algoritm = if accept_encoding_algoritms
            .iter()
            .find(|text| **text == "deflate")
            .is_some() {
                ENCODING::DEFLATE
            } else if accept_encoding_algoritms
            .iter()
            .find(|text| **text == "gzip")
            .is_some() {
                ENCODING::GZIP
            } else {
                return
            };

        let body = response.body_mut().to_bytes().await.map_err(
            |err| {
                log::warn_!("Encoder err: {}", err);
                return
            }
        ).unwrap();

        let copressed_body = match encoding_algoritm {
            ENCODING::DEFLATE => {
                response.set_header(Header::new("Content-Encoding", "deflate"));
                log::info_!("Encoder: apply deflate");
                deflate_bytes(&body)

                
            }
            ENCODING::GZIP => {
                response.set_header(Header::new("Content-Encoding", "gzip"));
                log::info_!("Encoder: apply gzip");
                deflate_bytes_gzip(&body)
            },
        };

        response.set_sized_body(copressed_body.len(), Cursor::new(copressed_body));
    }
}