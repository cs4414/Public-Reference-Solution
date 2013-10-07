//
// gash.rs
//
// Reference solution for PS2
// Running on Rust 0.7
//
// University of Virginia - cs4414 Fall 2013
// Weilin Xu, Purnam Jantrania, David Evans
// Version 0.1

use std::{io, run, os, path, uint, libc};
use std::task;


fn handle_cmd(cmd_line: &str, pipe_in: libc::c_int, pipe_out: libc::c_int, pipe_err: libc::c_int) {
    let mut out_fd = pipe_out;
    let mut in_fd = pipe_in;
    let err_fd = pipe_err;
    
    let mut argv: ~[~str] = cmd_line.split_iter(' ').filter(|&x| x != "").transform(|x| x.to_owned()).collect();
   
    let mut i = 0;
    while (i < argv.len()) {
        if (argv[i] == ~">") {
            argv.remove(i);
            let fpath = argv.remove(i);
            unsafe {
                out_fd = os::as_c_charp(fpath.to_str(), |pathbuf| {
                            os::as_c_charp("w", |modebuf| 
                            libc::fileno(libc::fopen(pathbuf, modebuf))
                    )});
            }
        } else if (argv[i] == ~"<") {
            argv.remove(i);
            let fpath = argv.remove(i);
            unsafe {
                in_fd = os::as_c_charp(fpath.to_str(), |pathbuf| {
                            os::as_c_charp("r", |modebuf| 
                            libc::fileno(libc::fopen(pathbuf, modebuf))
                    )});
            }
        }
        i += 1;
    }
    
    if argv.len() > 0 {
        let program = argv.remove(0);
        match program {
            ~"help"     => {println("This is a new shell implemented in Rust!")}
            ~"cd"       => {if argv.len()>0 {os::change_dir(&path::PosixPath(argv[0]));}}
            //global variable?
            //~"history"  => {for uint::range(0, history.len()) |i| {println(fmt!("%5u %s", i+1, history[i]));}}
            ~"exit"     => {unsafe{libc::exit(0);}}
            _           => {let mut prog = run::Process::new(program, argv, run::ProcessOptions {
                                                                                        env: None,
                                                                                        dir: None,
                                                                                        in_fd: Some(in_fd),
                                                                                        out_fd: Some(out_fd),
                                                                                        err_fd: Some(err_fd)
                                                                                    });
                             prog.finish();
                             // close the pipes after process terminates.
                             if in_fd != 0 {os::close(in_fd);}
                             if out_fd != 1 {os::close(out_fd);}
                             if err_fd != 2 {os::close(err_fd);}
                            }
        }//match 
    }//if
}

fn handle_cmdline(cmd_line:&str, bg_flag:bool)
{
    // handle pipes
    let progs: ~[~str] = cmd_line.split_str_iter("|").transform(|x| x.to_owned()).collect();
    
    let mut pipes = ~[];
    for uint::range(0, progs.len()-1) |_|{
        pipes.push(os::pipe());
    }
        
    if progs.len() == 1 {
        if bg_flag == false { handle_cmd(progs[0], 0, 1, 2); }
        else {task::spawn_sched(task::SingleThreaded, ||{handle_cmd(progs[0], 0, 1, 2)});}
    } else {
        for uint::range(0, progs.len()) |i| {
            let prog = progs[i].to_owned();
            
            if i == 0 {
                let pipe_i = pipes[i];
                task::spawn_sched(task::SingleThreaded, ||{handle_cmd(prog, 0, pipe_i.out, 2)});
            } else if i == progs.len() - 1 {
                let pipe_i_1 = pipes[i-1];
                if bg_flag == true {
                    task::spawn_sched(task::SingleThreaded, ||{handle_cmd(prog, pipe_i_1.in, 1, 2)});
                } else {
                    handle_cmd(prog, pipe_i_1.in, 1, 2);
                }
            } else {
                let pipe_i = pipes[i];
                let pipe_i_1 = pipes[i-1];
                task::spawn_sched(task::SingleThreaded, ||{handle_cmd(prog, pipe_i_1.in, pipe_i.out, 2)});
            }
        }
    }
}

fn main() {
    static CMD_PROMPT: &'static str = "gash > ";
    let mut history: ~[~str] = ~[];
    
    loop {
        print(CMD_PROMPT);
        
        let mut cmd_line = io::stdin().read_line();
        history.push(copy(cmd_line));
        let mut bg_flag = false;
        let amp_pos = cmd_line.find('&');
        if amp_pos != None {
            cmd_line = cmd_line.slice_to(amp_pos.get()).to_owned();
            bg_flag = true;
        }
        
        if cmd_line == ~"exit" {
            break;
        } else if cmd_line == ~"history" {
            for uint::range(0, history.len()) |i| {
                println(fmt!("%5u %s", i+1, history[i]));
            }
        } else {
            handle_cmdline(cmd_line, bg_flag);
        }
    }
}
