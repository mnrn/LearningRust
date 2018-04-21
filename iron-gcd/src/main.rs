extern crate iron;
#[macro_use] extern crate mime;  // We plan to use macros exported by this crate.

use iron::prelude::*;  // All the public names of the iron::prelude module directly visible in our own code.
use iron::status;

fn main() {
    println!("Serving on http://localhost:3000...");

    // Call Iron::new to create a server, and then sets it listening on TCP port 3000 on the local machine.
    // We pass the get_form functin ot Iron::new, indicating that the server should use that function to handle all requests;
    Iron::new(get_form).http("localhost:3000").unwrap();
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
        <input type="text" name="m"/>
        <botton type="submit">Compute GCD</button>
    </form>
    "#);

    // Some successful Response
    Ok(response)
}
