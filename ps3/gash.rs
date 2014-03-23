//
// gash module for ps2 and ps3
//
// University of Virginia - cs4414 Spring 2014
// Weilin Xu, David Evans
// Version 0.1
//

use std::{io, run, os, libc, path};
use std::io::pipe::PipeStream;

pub fn run_cmdline(cmd_line: &str) -> ~str{
    let cmd_line: ~str = cmd_line.trim().to_owned();
    
    // handle pipelines 
    let progs: ~[~str] =
        cmd_line.split('|').filter_map(|x| if x != "" { Some(x.to_owned()) } else { None }).to_owned_vec();
    
    let mut pipes: ~[os::Pipe] = ~[];
    
    // create pipes
    pipes.push(os::Pipe { input: 0, out: 0 }); // first pipe is standard input
    for _ in range(0, progs.len() - 1) {
        pipes.push(os::pipe());
    }
    pipes.push(os::pipe()); // last is not necessarily the standard output, for ps3
    
    for i in range(0, progs.len()) {
        run_single_cmd(progs[i], pipes[i].input, pipes[i+1].out, 2, 
                            if (i == progs.len() - 1) { false } else { true }); // all in bg except possibly last one
    }
    // read output from the last pipe.input.
     
    let mut pipe_stream = PipeStream::open(pipes[progs.len()].input);
    let content = pipe_stream.read_to_str();
    return content;
}

// run a single command line, probably with redirection sign >, definitly without pipelines | and background sign &.
fn run_single_cmd(cmd_line: &str, pipe_in: libc::c_int, pipe_out: libc::c_int, pipe_err: libc::c_int, bg: bool) {
    let mut argv = parse_argv(cmd_line);

    if argv.len() <= 0 {
        // empty command line
        return;
    }
    
    let mut out_fd = pipe_out;
    let mut in_fd = pipe_in;
    let err_fd = pipe_err;
    
    let mut i = 0;
    
    while (i < argv.len()) {
        if (argv[i] == ~">") {
            argv.remove(i);
            out_fd = get_fd(argv.remove(i), "w");
        } else if (argv[i] == ~"<") {
            argv.remove(i);
            in_fd = get_fd(argv.remove(i), "r");
        }
        i += 1;
    }
    
    let out_fd = out_fd;
    let in_fd = in_fd;
    
    if argv.len() <= 0 {
        // invalid command line
        return;
    }
    
    let program = argv.remove(0);
    match program {
        ~"cd"       => { if argv.len()>0 { os::change_dir(&path::Path::new(argv[0])); } }
        _           => { if !cmd_exists(program) {
                             println!("{:s}: command not found", program);
                             return;
                         } else {
                             // To see debug! outputs set the RUST_LOG environment variable, e.g.: export RUST_LOG="gash=debug" 
                             debug!("Program: {:s}, in_fd: {:d}, out_fd: {:d}, err_fd: {:d}", program, in_fd, out_fd, err_fd);
                             let opt_prog = run::Process::new(program, argv, 
                                                              run::ProcessOptions { env: None, dir: None,
                                                                                    in_fd: Some(in_fd), out_fd: Some(out_fd), err_fd: Some(err_fd)
                                                                                  });
                                
                             let mut prog = opt_prog.expect("Error: creating process error.");
                             if in_fd != 0 {os::close(in_fd);}
                             if out_fd != 1 {os::close(out_fd);}
                             if err_fd != 2 {os::close(err_fd);}

                             if !bg {
                                 prog.finish();
                                 io::stdio::flush();
                                 debug!("Terminated fg program: {:}", program);
                             } else {
                                 let (p_port, p_chan) = Chan::new();
                                 p_chan.send(prog);
                                 spawn(proc() {
                                    let mut prog: run::Process = p_port.recv();
                                       
                                    prog.finish(); 
                                    io::stdio::flush();
                                    debug!("Terminated bg program: {:}", program);
                                 });
                            }
                        }
                  }
    } // match program
} // run_single_cmd
    
// input: a single command line
// output: a vector of arguments. The program name is put in the first position.
// notes: arguments can be separated by space(s), ""  
fn parse_argv(cmd_line: &str) -> ~[~str] {
    let mut argv: ~[~str] = ~[];
    let group: ~[~str] = cmd_line.split('\"').filter_map(|x| if x != "" { Some(x.to_owned()) } else { None }).to_owned_vec();

    for i in range(0, group.len()) {            
        if i % 2 == 0 { // split by " "
            argv.push_all_move(group[i].split(' ').filter_map(|x| if x != "" { Some(x.to_owned()) } else { None }).to_owned_vec());
        } else {
            argv.push(group[i].clone());
        }
    }
    
    argv
}

fn cmd_exists(cmd_path: &str) -> bool {
    run::process_output("which", [cmd_path.to_owned()]).expect("exit code error.").status.success()
}

fn get_fd(fpath: &str, mode: &str) -> libc::c_int {
    unsafe {
        let fpathbuf = fpath.to_c_str().unwrap();
        let modebuf = mode.to_c_str().unwrap();
        return libc::fileno(libc::fopen(fpathbuf, modebuf));
    }
}

#[cfg(test)]
mod tests {
    use gash;
    #[test]
    fn test_run_cmdline() {
        let ret_str = run_cmdline("echo -e \"abc\nxyz\" | grep ab");
        println(ret_str);
        assert_eq!(ret_str, ~"abc");
    }
}
