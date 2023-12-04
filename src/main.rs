//mod config;
mod config;
use config::Config;
mod pty_process;
use regex::Regex;
mod select_process;
mod utilities;
use libc::{c_void, cfmakeraw, setsid, size_t, TCSANOW, TIOCGWINSZ, TIOCSCTTY, TIOCSWINSZ};
use nix::libc::{ioctl, STDERR_FILENO, STDIN_FILENO, STDOUT_FILENO};
use nix::unistd::{close, fork, ForkResult};

use std::os::unix::io::AsRawFd;
use std::os::unix::process::CommandExt;
use std::process::{exit, Command};
use std::time;
extern crate libc;


fn extract_pts_num(text: &str) -> Option<usize> {
    let re = Regex::new(r"/dev/pts/(\d+)").unwrap();
    re.captures(text).and_then(|caps| caps.get(1).map(|num| num.as_str().parse().unwrap()))
}
fn _clear_input(buffn: usize, pty: &mut pty_process::PTY) {
    for _bs in 0..buffn {
        let send_bs = String::from("\x08");
        let ptr = send_bs.as_ptr() as *const c_void;
        let len = send_bs.len() as size_t;
        unsafe {
            libc::write(pty.fdm.as_raw_fd(), ptr, len);
        }
    }
}

fn main() -> () {
    let mut pty_num = 0;
    let mut lua = hlua::Lua::new();
    lua.openlibs();
    lua.set(
        "print",
        hlua::function1(move |a: String| {
            println!("\n\rprint says: {}", a);
        }),
    );

    match Config::new() {
        Ok(config) => {
            let (init_type, _init_trigger, init_code) = Config::extract_init_values(config);
            // println!( "[Init]\nType: {}\nTrigger: {}\nCode:\n{}", init_type, init_trigger, init_code  );
            if init_type == "lua" {
                lua.execute::<()>(&init_code.to_string()).unwrap();
            }
        }
        Err(e) => {
            eprintln!("Error loading config: {}", e);
        }
    }

    unsafe {
        libc::tcgetattr(STDIN_FILENO, &mut pty_process::SAVE_TERM);
        libc::atexit(pty_process::restore_term);
    };

    if pty_process::install_signal_handler() == false {
        std::process::exit(1)
    }

    // get window dimensions
    let w = pty_process::Winsize {
        ws_row: 0,
        ws_col: 0,
        ws_xpixel: 0,
        ws_ypixel: 0,
    };
    unsafe { ioctl(STDOUT_FILENO, TIOCGWINSZ, &w) };

    // Open a master pty
    let pty = pty_process::new();
    match unsafe { fork().expect("Failed during fork()") } {
        ForkResult::Parent { .. } => {
            // This is the parent process
            unsafe {
                let mut new_term: libc::termios = pty_process::SAVE_TERM.clone();
                cfmakeraw(&mut new_term);
                libc::tcsetattr(STDIN_FILENO, TCSANOW, &new_term);
            }
            // Close the slave pty file descriptor
            close(pty.fds).expect("Error during closing the pty.fds in the child match arm");
            //let mut _read_buffer: [u8; 256] = [0; 256];
            
            let mut buffer: [u8; 256] = [0; 256];
            let mut buffn = 0;
        

            loop {
                let mut fd_set = select_process::FdSet::new();
                fd_set.set(0); // we want to observe the keyboard
                fd_set.set(pty.fdm.as_raw_fd()); // we want to observe output from the PTY

                let max_fd = pty.fdm.as_raw_fd(); // HACK HACK HACK
                match select_process::select( max_fd + 1,
                              Some(&mut fd_set),                                // read
                              None,                                             // write
                              None,                                             // error
                              Some(&select_process::make_timeval(time::Duration::new(60,0))), ) // timeout (sec, nsec)
                {
                    Ok(_) => {
                        let range = std::ops::Range { start: 0, end: max_fd + 1, };
                        for i in range {
                            if (fd_set).is_set(i) {
                                if 0==i {  // read from the keyboard
                                     
                                    let mut read_buf: [u8; 24] = [0; 24];
                                    let num_read = pty_process::fd_read(0, &mut read_buf) as i64;
                                    if num_read>0 {
                                        unsafe {
                                            if (1==num_read) && (read_buf[0].is_ascii()) {
                                                if read_buf[0]==0x7F || read_buf[0] == 0x8 {
                                                    pty_process::replace_last_element_greater_than_zero(&mut buffer);
                                                    let mut _user_string = String::from_utf8_lossy(&buffer[..buffn]);
                                                    //println!("\n\rPoString::from_utf8_lossy(&buffer[..buffn]) --  {} ", _user_string);

                                                    if buffn > 0{
                                                        buffn -= 1;
                                                    }
                                                    let ptr = read_buf.as_ptr() as *const c_void;
                                                    let len = 1 as size_t;
                                                    libc::write(pty.fdm.as_raw_fd(), ptr, len);
                                                } else if read_buf[0]==0xd {  // SEND CR , before sending CR, check if we have a special command
                                                    // ! foundme
                                                    //let output = String::new();
                                                    let mut _user_string = String::from_utf8_lossy(&buffer[..buffn]);
                                                    if _user_string.contains("get") {
                                                        for _bs in 0..buffn {
                                                            let send_bs = String::from("\x08");
                                                            let ptr = send_bs.as_ptr() as *const c_void;
                                                            let len = send_bs.len() as size_t;
                                                            
                                                                libc::write(pty.fdm.as_raw_fd(), ptr, len);
                                                            
                                                        }
                                                        let send_bs = String::from("tty\r");
                                                        let ptr = send_bs.as_ptr() as *const c_void;
                                                        let len = send_bs.len() as size_t;
                                                        libc::write(pty.fdm.as_raw_fd(), ptr, len);
                                                        let mut xxxx = [0; 256]; // Create a buffer to store the data
                                                        let mut output = String::new(); // Create a string to store the output
                                                        loop {
                                                            let mut _user_string = String::from_utf8_lossy(&xxxx[..buffn]);
                                                            let bytes_read = libc::read(pty.fdm.as_raw_fd(), xxxx.as_mut_ptr() as *mut c_void, xxxx.len());
                                                            if bytes_read > 0 {
                                                                let data = &xxxx[..bytes_read as usize];
                                                                output.push_str(&String::from_utf8_lossy(data)); // Append the data to the output string
                                                                //println!("\n\routput : {}", output);
                                                                //println!("{:?}", data);
                                                                let mut found = false; // Flag to indicate if the byte 13 has been found
                                                                for byte in data.iter() {
                                                                    // TODO DEBUG ME PLEASE ;)
                                                                    //println!("\r{:?}", byte);
                                                                
                                                                    if *byte == b'\r' || *byte == b'\n' || *byte == 8 ||  *byte == b'\t'{
                                                                        found = true;
                                                                        for _ in 0..buffn {
                                                                            pty_process::replace_last_element_greater_than_zero(&mut xxxx);
                                                                            let mut _user_string = String::from_utf8_lossy(&xxxx[..buffn]);
                                                                            //println!("\n\rPoString::from_utf8_lossy(&buffer[..buffn]) --  {} ", _user_string);
                                                                            
                                                                            if buffn > 0{
                                                                                buffn -= 1;
                                                                            }
                                                                        }
                                                                        break; // Exit the loop if the output contains "src"
                                                                    }
                                                                }
                                                                if !found && bytes_read < xxxx.len() as isize {
                                                                    break; 
                                                                }
                                                            } else {
                                                                break; // Exit the loop if no data was read
                                                            }
                                                        }
                                                        for _ in 0..buffn {
                                                            pty_process::replace_last_element_greater_than_zero(&mut buffer);
                                                            let mut _user_string = String::from_utf8_lossy(&buffer[..buffn]);
                                                            //println!("\n\rPoString::from_utf8_lossy(&buffer[..buffn]) --  {} ", _user_string);
                                                            
                                                            if buffn > 0{
                                                                buffn -= 1;
                                                            }
                                                        }
                                                        //println!("\n\rbuffer : {}", _user_string);
                                                        //let pts_output = output.trim().to_string();
                                                        //println!("\n\rxxxx{}", pts_output);
                                                        //println!("\r");
                                                        let send_cr = String::from("\r");
                                                        let ptr = send_cr.as_ptr() as *const c_void;
                                                        let len = 1 as size_t;
                                                        libc::write(pty.fdm.as_raw_fd(), ptr, len);
                                                        let pts_output = output.trim().to_string();
                                                        //println!("\rpts_output : \r{} ", pts_output);
                                                        match extract_pts_num(&pts_output) {
                                                            Some(num) => {
                                                                    pty_num = num;
                                                                    //println!("\r\npts number: {}", pty_num);
                                                                        //println!("\rpts number: {}", num);
                                                            },
                                                            None => {
                                                                //println!("\rpts number not found");
                                                                // Do nothing
                                                            }
                                                        }
                                                        //println!("{}", pts_output);

                                                        
                                                        // ! END
                                                    // ! send_keys(trigger, command)...It now works, 

                                                    } else if  _user_string.contains("which"){
                                                        // for _bs in 0..buffn {
                                                        //     let send_bs = String::from("\x08");
                                                        //         let ptr = send_bs.as_ptr() as *const c_void;
                                                        //         let len = send_bs.len() as size_t;
                                                        //         unsafe {
                                                        //             libc::write(pty.fdm.as_raw_fd(), ptr, len);
                                                        //         }
                                                        // }
                                                        let send_bs = String::from("\r");
                                                        let ptr = send_bs.as_ptr() as *const c_void;
                                                        let len = send_bs.len() as size_t;
                                                        libc::write(pty.fdm.as_raw_fd(), ptr, len);

                                                        let fd = pty_num;
                                                        let path = format!("/dev/pts/{}", fd);
                                                        let output = Command::new("fuser")
                                                            .arg("-v")
                                                            .arg(&path)
                                                            .output()
                                                            .expect("failed to execute fuser command");

                                                        let stdout = String::from_utf8_lossy(&output.stdout);
                                                        //let stderr = String::from_utf8_lossy(&output.stderr);
                                                        let curr_running_program_pid = stdout.split(' ').last().unwrap();   
                                                        //println!("\r\nstdout: {:?}", curr_running_program_pid);

                                                        let _output = Command::new("ps")
                                                            .arg("-p")
                                                            .arg(&curr_running_program_pid)
                                                            .output()
                                                            .expect("failed to execute fuser command");
                                                        let stdout = String::from_utf8_lossy(&output.stdout);
                                                        println!("\r\ncurrently running : {:?}", stdout.trim().split(' ').last().unwrap());
                                                        for _ in 0..5 {
                                                            pty_process::replace_last_element_greater_than_zero(&mut buffer);
                                                            let mut _user_string = String::from_utf8_lossy(&buffer[..buffn]);
                                                            //println!("\n\rPoString::from_utf8_lossy(&buffer[..buffn]) --  {} ", _user_string);
                                                            
                                                            if buffn > 0{
                                                                buffn -= 1;
                                                            }
                                                        }
                                                    } else if _user_string.contains("lk") {
                                                        for _bs in 0..buffn {
                                                            let send_bs = String::from("\x08");
                                                            let ptr = send_bs.as_ptr() as *const c_void;
                                                            let len = send_bs.len() as size_t;
                                                           
                                                                libc::write(pty.fdm.as_raw_fd(), ptr, len);
                                                            
                                                        }
                                                        let send_bs = String::from("ls\r");
                                                        let ptr = send_bs.as_ptr() as *const c_void;
                                                        let len = send_bs.len() as size_t;
                                                        libc::write(pty.fdm.as_raw_fd(), ptr, len);
                                                        let mut xxxx = [0; 256]; // Create a buffer to store the data
                                                        let mut output = String::new(); // Create a string to store the output
                                                        loop {
                                                            let mut _user_string = String::from_utf8_lossy(&xxxx[..buffn]);
                                                            let bytes_read = libc::read(pty.fdm.as_raw_fd(), xxxx.as_mut_ptr() as *mut c_void, xxxx.len());
                                                            if bytes_read > 0 {
                                                                let data = &xxxx[..bytes_read as usize];
                                                                output.push_str(&String::from_utf8_lossy(data)); // Append the data to the output string
                                                                //println!("\n\routput : {}", output);
                                                                //println!("{:?}", data);
                                                                let mut found = false; // Flag to indicate if the byte 13 has been found
                                                                for byte in data.iter() {
                                                                    // TODO DEBUG ME PLEASE ;)
                                                                    //println!("\r{:?}", byte);
                                                                
                                                                    if *byte == b'\r' || *byte==13 || *byte == b'\n' || *byte == 8 ||  *byte == b'\t'{
                                                                        found = true;
                                                                        for _ in 0..3 {
                                                                            pty_process::replace_last_element_greater_than_zero(&mut xxxx);
                                                                            let mut _user_string = String::from_utf8_lossy(&xxxx[..buffn]);
                                                                            //println!("\n\rPoString::from_utf8_lossy(&buffer[..buffn]) --  {} ", _user_string);
                                                                            
                                                                            if buffn > 0{
                                                                                buffn -= 1;
                                                                            }
                                                                        }
                                                                        break; // Exit the loop if the output contains "src"
                                                                    }
                                                                }
                                                                if !found && bytes_read < xxxx.len() as isize {
                                                                    break; // Exit the loop if the end of the data slice has been reached and no 13 byte has been found
                                                                }
                                                            } else {
                                                                break; // Exit the loop if no data was read
                                                            }
                                                        }
                                                        for _ in 0..buffn {
                                                            pty_process::replace_last_element_greater_than_zero(&mut buffer);
                                                            let mut _user_string = String::from_utf8_lossy(&buffer[..buffn]);
                                                            //println!("\n\rPoString::from_utf8_lossy(&buffer[..buffn]) --  {} ", _user_string);
                                                            
                                                            if buffn > 0{
                                                                buffn -= 1;
                                                            }
                                                        }
                                                        //println!("\n\rbuffer : {}", _user_string);
                                                        //let pts_output = output.trim().to_string();
                                                        //println!("\n\rxxxx{}", pts_output);
                                                        //println!("\r");
                                                        let send_cr = String::from("\r");
                                                        let ptr = send_cr.as_ptr() as *const c_void;
                                                        let len = 1 as size_t;
                                                        libc::write(pty.fdm.as_raw_fd(), ptr, len);
                                                        let pts_output = output.trim().to_string();

                                                        //println!("{}", pts_output);
                                                        // ! END
                                                    }else {
                                                        let send_cr = String::from("\r");
                                                        let ptr = send_cr.as_ptr() as *const c_void;
                                                        let len = 1 as size_t;
                                                        libc::write(pty.fdm.as_raw_fd(), ptr, len);
                                                        buffn = 0;
                                                    }
                                                    

                                                } else {
                                                    buffer[buffn] = read_buf[0];
                                                    buffn += 1;
                                                    let _user_string = String::from_utf8_lossy(&buffer[..buffn]);
                                                    //println!("\n\r normal --  {} ", _user_string);   
                                                    let ptr = read_buf.as_ptr() as *const c_void;
                                                    let len = 1 as size_t;
                                                    libc::write(pty.fdm.as_raw_fd(), ptr, len);
                                                }
                                            } else { //if the input is non-ascii i.e terminal gui apps
                                                let ptr = read_buf.as_mut_ptr() as *const c_void;
                                                let len = num_read as size_t;
                                                libc::write(pty.fdm.as_raw_fd(), ptr, len);
                                            }
                                        }
                                    } else {
                                        exit(0);
                                    }
                                } else if pty.fdm.as_raw_fd()==i {
                                    let mut read_buf: [u8; 256] = [0; 256];
                                    let num_read = pty_process::fd_read(pty.fdm.as_raw_fd(), &mut read_buf) as i64;
                                    if num_read>0 {
                                        unsafe {
                                            let ptr = read_buf.as_mut_ptr() as *const c_void;
                                            let len = num_read as size_t;
                                            libc::write(0, ptr, len);
                                        }
                                    } else {
                                        exit(0);
                                    }
                                }
                            } else {
                                // timeout
                            }
                        }
                    }
                    Err(_) => {
                        // select was interrupted - adjust terminal window size
                        unsafe {
                            let w = pty_process::Winsize { ws_row: 0, ws_col: 0, ws_xpixel: 0, ws_ypixel: 0 };
                            ioctl(STDOUT_FILENO, TIOCGWINSZ, &w);
                            ioctl(pty.fdm.as_raw_fd(), TIOCSWINSZ, &w);
                        }
                    }
                }
            }
        }
        ForkResult::Child => {
            // This is the child process
            drop(pty.fdm);
            unsafe {
                libc::close(STDIN_FILENO);
                libc::close(STDOUT_FILENO);
                libc::close(STDERR_FILENO);
                libc::dup(pty.fds);
                libc::dup(pty.fds);
                libc::dup(pty.fds);
                libc::close(pty.fds);
                setsid();
                ioctl(0, TIOCSCTTY as u64, 1);
                ioctl(STDOUT_FILENO, TIOCSWINSZ, &w);
            }
            // ! this is a one-way street...
            //Command::new("/bin/echo").arg("XXX").exec();
            Command::new("/bin/bash").exec();
        }
    }
}
