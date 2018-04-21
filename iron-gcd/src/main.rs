extern crate iron;
#[macro_use] extern crate mime;  // We plan to use macros exported by this crate.
extern crate router;
extern crate urlencoded;

use std::str::FromStr;
use iron::prelude::*;  // All the public names of the iron::prelude module directly visible in our own code.
use iron::status;
use router::Router;
use urlencoded::UrlEncodedBody;

fn main() {
    // We create a Router, establish handler functions for two specific paths, and then pass this Router as theh request handler to Iron::new,
    // yielding a web server that consults the URL path to decide which handler function to call.
    let mut router = Router::new();
    router.get("/", get_form, "root");
    router.post("/gcd", post_gcd, "gcd");

    println!("Serving on http://localhost:3000...");

    // Call Iron::new to create a server, and then sets it listening on TCP port 3000 on the local machine.
    Iron::new(router).http("localhost:3000").unwrap();
}

fn post_gcd(request: &mut Request) -> IronResult<Response> {
    let mut response = Response::new();

    // Parse the request's body as a table mapping query parameter names to arrays of values
    // if this parse failes, it reports the error back to the client.
    // The ::<UrlEncodedBody> part of the method calls is a type parameter indicating which part of the Request get_ref should retrieve.
    // In this case, the UrlEncodedBody type refers to the body, parsed as a URL-encoded query string.
    let form_data = match request.get_ref::<UrlEncodedBody>() {
        Err(e) => {
            response.set_mut(status::BadRequest);
            response.set_mut(format!("Error parsing form data:{:?}\n", e));
            return Ok(response);
        }
        Ok(map) => map
    };

    // Within that table, it finds the value of the parameter named "n", which is where the HTML form places the numbers entered into the web page.
    // This value will be not a single string but a vector of strings, as query parameter names can be repeated.
    let unparsed_numbers = match form_data.get("n") {
        None => {
            response.set_mut(status::BadRequest);
            response.set_mut(format!("form data has no 'n' parameter\n"));
            return Ok(response);
        }
        Some(nums) => nums
    };

    // It walks the vector of strings, parsing each one as an unsigned 64-bit bumber, and returning an appropriate failure page if any of the string fail to parse.
    let mut numbers = Vec::new();
    for unparsed in unparsed_numbers {
        match u64::from_str(&unparsed) {
            Err(_) => {
                response.set_mut(status::BadRequest);
                response.set_mut(format!("Value for 'n' parameter not a number: {:?}\n", unparsed));
                return Ok(response);
            }
            Ok(n) => { 
                numbers.push(n);
            }
        }
    }

    // It computes the numbers' greatesst common divisor.
    let mut d = numbers[0];
    for m in &numbers[1..] {
        d = gcd(d, *m);
    }

    response.set_mut(status::Ok);  // passing status::Ok sets the HTTP status.
    response.set_mut(mime!(Text/Html; Charset=Utf8));  // passing the media type of content sets th Content-Type header;
    // passing a string sets the response body.
    response.set_mut(format!("The greatest common divisor of the numbers {:?} is <b>{}</b>\n", numbers, d));

    // Some successful Response
    Ok(response)
}

// The get_form function itself takes a mutable reference, written &mut, to a Request value representing the HTTP request we've been called to handle.
fn get_form(_request: &mut Request) -> IronResult<Response> {
    let mut response = Response::new();

    response.set_mut(status::Ok);  // passing status::Ok sets the HTTP status.
    response.set_mut(mime!(Text/Html; Charset=Utf8));  // passing the media type of content sets th Content-Type header;
    // passing a string sets the response body.
    response.set_mut(r#"
    <title>GCD Calculator</title>
    <form action="/gcd" method="post">
        <input type="text" name="n"/>
        <input type="text" name="n"/>
        <button type="submit">Compute GCD</button>
    </form>
    "#);

    // Some successful Response
    Ok(response)
}

// Calculate greatest common divisor.
fn gcd(mut n: u64, mut m: u64) ->u64 {
    assert!(n != 0 && m != 0);
    while m != 0 {
        if m < n {
            std::mem::swap(&mut n, &mut m);
        }
        m = m % n;
    }
    n
}

#[test]
fn test_gcd() {
    assert_eq!(gcd(14, 15), 1);
    assert_eq!(gcd( 2 * 3 * 5 * 11 * 17,
                    3 * 7 * 11 * 13 * 19),
                    3 * 11);
}
