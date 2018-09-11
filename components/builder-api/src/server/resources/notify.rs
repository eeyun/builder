// Copyright (c) 2018 Chef Software Inc. and/or applicable contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use actix_web::http::{Method, StatusCode};
use actix_web::{App, HttpRequest, HttpResponse};

use server::authorize::authorize_session;
use server::framework::headers;
use server::services::github;
use server::AppState;

pub struct Notify;

impl Notify {
    //
    // Route registration
    //
    pub fn register(app: App<AppState>) -> App<AppState> {
        app.route("/notify", Method::POST, notify)
    }
}

pub fn notify((req, body): (HttpRequest<AppState>, String)) -> HttpResponse {
    if let Err(err) = authorize_session(&req, None) {
        return err.into();
    }

    if req.headers().get(headers::XGITHUBEVENT).is_some() {
        match github::handle_event(req, body) {
            Ok(_) => HttpResponse::new(StatusCode::OK),
            Err(err) => err.into(),
        };
    }
    return HttpResponse::new(StatusCode::BAD_REQUEST);
}