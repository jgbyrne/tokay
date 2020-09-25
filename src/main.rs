use std::cell::RefCell;
use std::rc::Rc;

use std::io::prelude::*;

use ::tokay::reader::Reader;
use ::tokay::tokay::*;
use ::tokay::token::*;
use ::tokay::value::Value;
use ::tokay::{ccl, tokay, sequence, modifier, item};


fn main() {
    let s = "123+456".to_string();
    println!("{}", s);

    let mut program = Program::new();

    //trace_macros!(true);

    let mut program = tokay!(
        /*
        main {
            => (sub)
            => ("A")
        }

        sub {
            => (subsub)
            => ("B")
        }

        subsub {
            => (main)
        }

        x {
            => ("Y")  (main)
        }

        main {
            => ("Hello") ("World")
            => ("Hello") ("Rofl")
            => ("Doppel") (main)
            => (main) ("bla")
            => ("Hello") ("Ralf")
        }
        */

        main {
            => (expr)
            /*
            => (("hello") ((kle("world")) (|runtime| {
                let hello = runtime.get_capture(1).unwrap().borrow().to_string().unwrap();
                let world = runtime.get_capture(2).unwrap().borrow().to_string().unwrap();
        
                println!("{} {} {}", runtime.get_capture(0).unwrap().borrow().to_string().unwrap(), hello, world);
                Ok(Accept::Next)
            })))
            */
        }

        factor {
            => ("(") (expr) (")")
            => (int)
        }

        term {
            => (term) ("*") (factor)
            => (term) ("/") (factor)
            => (factor)
        }

        expr {
            => (expr) ("+") (term)
            => (expr) ("-") (term)
            => (term)
        }

        int {
            =>  ("x")
                /*
                (Token::Chars(ccl!['0'..='9']))
                (|runtime| {
                    //println!("{:?}", runtime.get_capture(0));

                    if let Some(i) = runtime.get_capture(1).unwrap().borrow().to_integer() {
                        Ok(Accept::Return(Some(Value::Integer(i).into_ref())))
                    }
                    else {
                        Err(Reject::Return)
                    }
                })
                */
        }
    );
    //trace_macros!(false);
    
    //let s = "42+3-1337/3*2  helloworldworldworldhellohelloworld 7*(2+5) world  666-600 3".to_string();
    let mut reader = Reader::new(
        Box::new(std::io::Cursor::new(s))
    );

    program.finalize();

    let mut runtime = Runtime::new(&program, &mut reader);
    let ret = program.run(&mut runtime);

    println!("{:?}", ret);

}
