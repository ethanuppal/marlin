#[no_mangle] extern "C" fn three( out : * mut u32){ let out = unsafe { & mut * out }; { * out = 3; } }
