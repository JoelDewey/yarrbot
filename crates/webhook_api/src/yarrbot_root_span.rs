use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::Error;
use tracing::Span;
use tracing_actix_web::{DefaultRootSpanBuilder, RootSpanBuilder};

pub struct YarrbotRootSpan;

impl RootSpanBuilder for YarrbotRootSpan {
    fn on_request_start(request: &ServiceRequest) -> Span {
        use tracing::field::Empty;

        tracing_actix_web::root_span!(request, webhook_arr_type = Empty, webhook_short_id = Empty,)
    }

    fn on_request_end<B>(span: Span, outcome: &Result<ServiceResponse<B>, Error>) {
        DefaultRootSpanBuilder::on_request_end(span, outcome);
    }
}
