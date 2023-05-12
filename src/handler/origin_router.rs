use std::collections::HashMap;
use trillium::{Conn, Handler, Info};

fn origin(conn: &Conn) -> String {
    let scheme = if conn.is_secure() { "https" } else { "http" };

    let host = conn.inner().host().unwrap_or_default().to_lowercase();

    format!("{scheme}://{host}")
}

#[derive(Default, Debug)]
pub struct OriginRouter {
    map: HashMap<String, Box<dyn Handler>>,
}

impl OriginRouter {
    fn handler(&self, conn: &Conn) -> Option<&dyn Handler> {
        self.map
            .get(&origin(conn))
            .map(|boxed_handler| &**boxed_handler)
    }

    /// Construct a new OriginRouter.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a handler to this origin router at the specified exact origin, returning self.
    /// See also [`add_handler`].
    pub fn with_handler(mut self, origin: &str, handler: impl Handler) -> Self {
        self.add_handler(origin, handler);
        self
    }

    /// Add a handler to this origin router at the specified exact origin.
    /// See also [`with_handler`] for chainability.
    pub fn add_handler(&mut self, origin: &str, handler: impl Handler) {
        self.map.insert(
            origin.to_lowercase().trim_end_matches('/').to_owned(),
            Box::new(handler),
        );
    }
}

#[trillium::async_trait]
impl Handler for OriginRouter {
    async fn run(&self, conn: Conn) -> Conn {
        if let Some(handler) = self.handler(&conn) {
            handler.run(conn).await
        } else {
            conn
        }
    }

    async fn before_send(&self, conn: Conn) -> Conn {
        if let Some(handler) = self.handler(&conn) {
            handler.before_send(conn).await
        } else {
            conn
        }
    }

    async fn init(&mut self, info: &mut Info) {
        for value in self.map.values_mut() {
            value.init(info).await
        }
    }
}

pub fn origin_router() -> OriginRouter {
    OriginRouter::new()
}
