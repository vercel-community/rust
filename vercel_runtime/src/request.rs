use lambda_http::http::{self, header::HeaderValue, HeaderMap, Method};
use lambda_http::Body;
use serde::de::{Deserializer, Error as DeError, MapAccess, Visitor};
use serde_derive::Deserialize;
use serde_json::Value;
use std::{borrow::Cow, fmt, mem};

/// Representation of a Vercel Lambda proxy event data
#[doc(hidden)]
#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub(crate) struct VercelRequest<'a> {
    pub(crate) host: Cow<'a, str>,
    pub(crate) path: Cow<'a, str>,
    #[serde(deserialize_with = "deserialize_method")]
    pub(crate) method: Method,
    #[serde(deserialize_with = "deserialize_headers")]
    pub(crate) headers: HeaderMap<HeaderValue>,
    pub(crate) body: Option<Cow<'a, str>>,
    pub(crate) encoding: Option<String>,
}

#[doc(hidden)]
#[derive(Deserialize, Debug, Default)]
pub struct VercelEvent<'a> {
    #[allow(dead_code)]
    #[serde(rename = "Action")]
    action: Cow<'a, str>,
    pub(crate) body: Cow<'a, str>,
}

fn deserialize_method<'de, D>(deserializer: D) -> Result<Method, D::Error>
where
    D: Deserializer<'de>,
{
    struct MethodVisitor;

    impl<'de> Visitor<'de> for MethodVisitor {
        type Value = Method;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(formatter, "a Method")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: DeError,
        {
            v.parse().map_err(E::custom)
        }
    }

    deserializer.deserialize_str(MethodVisitor)
}

fn parse_scalar(v: &serde_json::Value) -> Result<HeaderValue, Box<dyn std::error::Error>> {
    Ok(match v {
        Value::Null => HeaderValue::from_str("")?,
        Value::Bool(true) => HeaderValue::from_str("true")?,
        Value::Bool(false) => HeaderValue::from_str("false")?,
        Value::Number(n) => HeaderValue::from_str(&n.to_string())?,
        Value::String(s) => HeaderValue::from_str(s)?,
        Value::Object(_) | Value::Array(_) => {
            return Err(format!("expected scalar but got {:?}", v).into())
        }
    })
}

fn deserialize_headers<'de, D>(deserializer: D) -> Result<HeaderMap<HeaderValue>, D::Error>
where
    D: Deserializer<'de>,
{
    struct HeaderVisitor;

    impl<'de> Visitor<'de> for HeaderVisitor {
        type Value = HeaderMap<HeaderValue>;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(formatter, "a HeaderMap<HeaderValue>")
        }

        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: MapAccess<'de>,
        {
            let mut headers = http::HeaderMap::new();

            while let Some((key, value)) = map.next_entry::<&str, Value>()? {
                let header_name = key
                    .parse::<http::header::HeaderName>()
                    .map_err(A::Error::custom)?;

                match value {
                    Value::Object(o) => {
                        return Err(A::Error::custom(format!(
                            "unable to deserialize object inside headers: {:?}",
                            o
                        )))
                    }
                    Value::Array(values) => {
                        let str_vec_values = values
                            .iter()
                            .map(|v| {
                                let value = v.as_str().ok_or_else(|| {
                                    A::Error::custom(format!(
                                        "unable to stringify array value inside headers: {:?}",
                                        v
                                    ))
                                });

                                value
                            })
                            .collect::<Result<Vec<&str>, _>>()?;

                        let joined_values = str_vec_values.join(",");

                        headers.append(
                            &header_name,
                            HeaderValue::from_str(&joined_values).map_err(A::Error::custom)?,
                        );
                    }
                    Value::Number(_) | Value::Bool(_) | Value::String(_) | Value::Null => {
                        headers.append(
                            &header_name,
                            parse_scalar(&value).map_err(A::Error::custom)?,
                        );
                    }
                };
            }
            Ok(headers)
        }
    }

    deserializer.deserialize_map(HeaderVisitor)
}

impl<'a> From<VercelRequest<'a>> for http::Request<Body> {
    fn from(value: VercelRequest<'_>) -> Self {
        let VercelRequest {
            host,
            path,
            method,
            headers,
            body,
            encoding,
        } = value;

        // Build an http::Request<vercel_runtime::Body> from a vercel_runtime::VercelRequest
        let builder = http::Request::builder()
            .method(method)
            .uri(format!("https://{}{}", host, path));

        let mut req = builder
            .body(match (body, encoding) {
                (Some(ref b), Some(ref encoding)) if encoding == "base64" => {
                    // TODO: Document failure behavior
                    Body::from(::base64::decode(b.as_ref()).unwrap_or_default())
                }
                (Some(b), _) => Body::from(b.into_owned()),
                _ => Body::from(()),
            })
            .expect("failed to build request");

        // No builder method that sets headers in batch
        let _ = mem::replace(req.headers_mut(), headers);

        req
    }
}
