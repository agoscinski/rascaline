use std::os::raw::c_char;

use rascaline::Error;

pub unsafe fn copy_str_to_c(string: &str, buffer: *mut c_char, buflen: usize) -> Result<(), Error> {
    let size = std::cmp::min(string.len(), buflen - 1);
    if size < string.len() {
        return Err(Error::InvalidParameter(format!(
            "string buffer is not big enough: got space for {} characters, but we need to write {}",
            size, string.len()
        )))
    }

    std::ptr::copy(string.as_ptr(), buffer as *mut u8, size);
    // NULL-terminate the string
    buffer.add(size).write(0);
    Ok(())
}
