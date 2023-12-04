// if _user_string == "hi" {
//     for _bs in 0..buffn {
//         let send_bs = String::from("\x08");
//         let ptr = send_bs.as_ptr() as *const c_void;
//         let len = send_bs.len() as size_t;
//         unsafe {
//             libc::write(pty.fdm.as_raw_fd(), ptr, len);
//         }
//     }
//     let mut read_buffer = [0u8; BUFFER_SIZE];
//     let mut total_bytes_read = 0;
//     let mut temp_buff: [u8; 256] = [0; 256];
//     loop {
//         let bytes_read = unsafe {
//             libc::read(0, read_buffer.as_mut_ptr() as *mut c_void, 10)
//         };
//         if bytes_read <= 0 {
//             // Error or end of file
//             break;
//         }
//         for i in 0..bytes_read {
//             temp_buff[total_bytes_read] = read_buffer[i as usize];
//             total_bytes_read += 1;
//         }
//         if read_buffer[0] == 0xd {
//             let null = File::create("/dev/null").unwrap();
//             unsafe {
//                 dup2(null.as_raw_fd(), STDOUT_FILENO);
//                 dup2(null.as_raw_fd(), STDERR_FILENO);
//             }
//             let send_cmd = "ls\r";
//             let ptr = send_cmd.as_ptr() as *const c_void;
//             let len = send_cmd.len() as size_t;
//             unsafe {
//                 libc::write(pty.fds.as_raw_fd(), ptr, len);
//             }
//             let mut output = String::new();
//             loop {
//                 let mut buf = [0; 1024];
//                 let bytes_read = pty.fdm.read(&mut buf).unwrap();
//                 if bytes_read <= 0 {
//                     // Error or end of file
//                     break;
//                 }
//                 output.push_str(std::str::from_utf8(&buf[..bytes_read]).unwrap());
//             }
//             println!("Output of the 'ls' command:\n{}", output);
//             let stdout = File::open("/dev/stdout").unwrap();
//             let stderr = File::open("/dev/stderr").unwrap();
//             unsafe {
//                 dup2(stdout.as_raw_fd(), STDOUT_FILENO);
//                 dup2(stderr.as_raw_fd(), STDERR_FILENO);
//             }
//         }
//     }
// }
