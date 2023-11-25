use alloc::ffi::CString;
use core::task::Poll;
use embedded_io_async::{ErrorType, Read, Write};
use esp_idf_hal::io::EspIOError;
use esp_idf_sys::EspError;

pub struct TcpStream(*mut esp_idf_sys::esp_tls);

pub trait TcpConnect: Read + Write + Sized {
    async fn connect_http(url: &str, is_plain_tcp: bool) -> Result<Self, EspError>;
    async fn connect(host_name: &str, port: u16, is_plain_tcp: bool) -> Result<Self, EspError>;
}

impl TcpConnect for TcpStream {
    async fn connect_http(url: &str, is_plain_tcp: bool) -> Result<Self, EspError> {
        let conn = Self(unsafe { esp_idf_sys::esp_tls_init() });
        let result = {
            let tls = conn.0;
            core::future::poll_fn(|_ctx| {
                let c_url = CString::new(url).unwrap();
                let result = unsafe {
                    esp_idf_sys::esp_tls_conn_http_new_async(
                        c_url.as_ptr(),
                        &esp_idf_sys::esp_tls_cfg_t {
                            is_plain_tcp,
                            use_global_ca_store: true,
                            #[cfg(not(esp_idf_version = "4.3"))]
                            crt_bundle_attach: Some(esp_idf_sys::esp_crt_bundle_attach),
                            ..Default::default()
                        },
                        tls,
                    )
                };
                match result {
                    0 => Poll::Pending,
                    other => Poll::Ready(other),
                }
            })
            .await
        };
        match result {
            1 => Ok(conn),
            other => Err(EspError::from(other).unwrap()),
        }
    }

    async fn connect(host_name: &str, port: u16, is_plain_tcp: bool) -> Result<Self, EspError> {
        let conn = Self(unsafe { esp_idf_sys::esp_tls_init() });
        let result = {
            let tls = conn.0;
            core::future::poll_fn(|_ctx| {
                let c_host_name = CString::new(host_name).unwrap();
                let result = unsafe {
                    esp_idf_sys::esp_tls_conn_new_async(
                        c_host_name.as_ptr(),
                        c_host_name.as_bytes().len() as _,
                        port as _,
                        &esp_idf_sys::esp_tls_cfg_t {
                            is_plain_tcp,
                            use_global_ca_store: true,
                            #[cfg(not(esp_idf_version = "4.3"))]
                            crt_bundle_attach: Some(esp_idf_sys::esp_crt_bundle_attach),
                            ..Default::default()
                        },
                        tls,
                    )
                };
                match result {
                    0 => Poll::Pending,
                    other => Poll::Ready(other),
                }
            })
            .await
        };
        match result {
            1 => Ok(conn),
            other => Err(EspError::from(other).unwrap()),
        }
    }
}

impl ErrorType for TcpStream {
    type Error = EspIOError;
}

impl embedded_io_async::Read for TcpStream {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        let result = core::future::poll_fn(|_ctx| {
            match unsafe {
                esp_idf_sys::esp_tls_conn_read(self.0, buf.as_mut_ptr() as _, buf.len())
            } as i32
            {
                esp_idf_sys::ESP_TLS_ERR_SSL_WANT_READ => Poll::Pending,
                code => Poll::Ready(code),
            }
        })
        .await;
        match EspError::from(result) {
            Some(err) if result < 0 => Err(EspIOError(err)),
            _ => Ok(result as _),
        }
    }
}

impl embedded_io_async::Write for TcpStream {
    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        let result = core::future::poll_fn(|_ctx| {
            match unsafe { esp_idf_sys::esp_tls_conn_write(self.0, buf.as_ptr() as _, buf.len()) }
                as i32
            {
                esp_idf_sys::ESP_TLS_ERR_SSL_WANT_WRITE => Poll::Pending,
                code => Poll::Ready(code),
            }
        })
        .await;
        match EspError::from(result) {
            Some(err) if result < 0 => Err(EspIOError(err)),
            _ => Ok(result as _),
        }
    }
}

impl Drop for TcpStream {
    fn drop(&mut self) {
        unsafe {
            esp_idf_sys::esp_tls_conn_destroy(self.0);
        }
    }
}
