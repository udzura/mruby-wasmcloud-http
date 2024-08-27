use mrubyedge::{mrb_helper, vm::RObject};
use std::{cell::RefCell, rc::Rc};

wit_bindgen::generate!({ generate_all });

use exports::wasi::http::incoming_handler::Guest;
use wasi::http::types::*;

struct HttpServer;

impl Guest for HttpServer {
    fn handle(_request: IncomingRequest, response_out: ResponseOutparam) {
	let write_response = |body: &str| {
                let response = OutgoingResponse::new(Fields::new());
                response.set_status_code(200).unwrap();
                let response_body = response.body().unwrap();
                ResponseOutparam::set(response_out, Ok(response));
                response_body
                    .write()
                    .unwrap()
                    .blocking_write_and_flush(body.as_bytes())
                    .unwrap();
                OutgoingBody::finish(response_body, None).expect("failed to finish response body");
	};
	
        let bin = include_bytes!("./fib.mrb");
        let rite = mrubyedge::rite::load(bin).unwrap();
        let mut vm = mrubyedge::vm::VM::open(rite);
        vm.prelude().unwrap();
        vm.eval_insn().unwrap();
        let objclass_sym = vm.target_class.unwrap() as usize;
        let top_self = RObject::RInstance {
            data: Rc::new(RefCell::new(Box::new(()))),
            class_index: objclass_sym,
        };
        let args = vec![Rc::new(RObject::RInteger(15))];
        match mrb_helper::mrb_funcall(&mut vm, &top_self, "fib".to_string(), &args) {
            Ok(retval) => {
		let val: i32 = retval.as_ref().try_into().unwrap();
                let body = format!("fib(15) = {}\n", val);
		write_response(&body);
            }
            Err(ex) => {
                dbg!(ex);
                let body = format!("Error = {:?}\n", ex);
		write_response(&body);
            }
        };
    }
}

export!(HttpServer);
