/* 
 * WORKING TIME CALCULATOR
 * 
 * 
 * Author: Korbinian Maier
 * Date: 10/03/2018
 *
 * TODO:
 * - file as path, path as constant or config file
 * - error handling in file writing functions
 * - print current content of file
 * 
 * 
 */

/****  CRATES   ****/
extern crate time;

/****  MODULES  ****/
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;
use std::fmt;
use std::env;

/**** CONSTANTS ****/
static VERSION: &'static str = "0.3.0";

/****  STRUCTS  ****/
#[derive(Debug)]
struct TimeDuration {
    /// Year: yyyy [0..9999]
    year: i32, 
    /// Month: mm [1..12]
    month: i32,
    /// Day: dd [1..31]
    day: i32,
    /// Duration: float
    duration: f64,
}

impl TimeDuration {
    /// conversion to string: yyyy,mm,dd,hh.mm
    fn to_string( &self) -> String {
        let mut res;
        res = self.year.to_string();
        res.push_str( ",");
        res.push_str( &format!("{:02}", &self.month) );
        res.push_str(",");
        res.push_str( &format!("{:02}", &self.day) );
        res.push_str(",");
        if &self.duration > &9.0 { // add leading zero to float if smaller 10
            res.push_str( &format!("{:.02}", &self.duration) );
        } else {
            res.push_str( &format!("0{:.02}", &self.duration) );
        }
        res.to_owned()
    }
}

impl fmt::Display for TimeDuration {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let hour = &self.duration.trunc();
        let min = ((&self.duration - hour) * 60.0) as i32;
        write!(f, "{:02}.{:02}.{:04}, Duration: {:02}:{:02}h", &self.day, &self.month, &self.year, hour, min)
    }
}

fn main() {

    // create path to file, file has to be in current directory
    let mut path = match env::current_dir() {
        Ok(path)    => path,
        Err(_)      => PathBuf::from("~/"),
    };
    path.push("time_storage.txt");  // TODO: put into config file

    // input arguments processing
    let args: Vec<String> = env::args().collect();
    
    if args.len() > 1 {
        match args[1].to_string().as_ref() {
            "start" => write_start_to_file( &path ),
            "stop"  => write_time_to_file( calc_duration(get_start_time( &path )) , &path),
            "show"  => {
                print_stats( &path );
                calc_stats( &path )
            },
            "version"=> println!("Version: {}", VERSION),
            _       => println!("no implemented. print help or similar"),
        }
    } else {
        println!("No arguments. Use 'start' or 'stop'.");
    }

}


/* Write time to file
 * Format: yyyy,mm,dd,duration\n
 */
fn write_time_to_file( dat: TimeDuration, file_name: &PathBuf) {

    // Open file and read content into string, TODO: Handle error properly
    // TODO: add case for non existing file
    let mut content = match read_file_to_string( &file_name) {
        Ok(string)=> string,
        Err(e)    => e,
    }; 

    // add TimeDuration to end of content
    content.push_str( &dat.to_string() );
    content.push_str("\n");

    // create new file with same name, FIXME: can't write to opened file, have to create new one
    let mut storage = File::create(file_name).expect("Could not create new file for writing.");

    // Write whole content to file
    // TODO: return error message
    storage.write( content.as_bytes()).expect("Write time fo file: Error");
}

/* Calculates duration from passed TimeDuration and current time
 * returns TimeDuration with the calculated duration and current date
 */
fn calc_duration( duration: f64 ) -> TimeDuration {

    // get current time
    let now = time::now();
    // generate result type with current year, month and day
    let mut res = TimeDuration{ year: now.tm_year + 1900,
                                month: now.tm_mon + 1,
                                day: now.tm_mday,
                                duration: 0.};
    
    // calculate duration
    res.duration =  now.tm_hour as f64
                        + (now.tm_min as f64)/60.0
                        - duration;
    let hour = res.duration.trunc();
    let min = ((res.duration - hour) * 60.0) as i32;
    println!("Calculated Duration: {:02}:{:02}h", hour, min);
    res
}

/* Get starting time from file
 * Should be in last line 
 */
fn get_start_time( file_name: &PathBuf) -> f64 {

    // open file and read content to string
    let content = match read_file_to_string( &file_name) {
        Ok(string)=> string,
        Err(e)    => e,
    }; 

    // get last line
    let split =  match content.lines().last() {
        Some(split)   => split,
        None    => "None",
    };

    // get last number of last line, contains starting time
    let dur = match split.split(',').last() {
        Some(dur)   => dur,
        None        => "0",
    };

    // delete last line, no longer needed
    delete_start_in_file( &file_name );
    
    dur.parse().unwrap()
}

/* Write starting time (which is current time) to file
 * as TimeDuration with duration is hour + min/60
 */
fn write_start_to_file( file_name: &PathBuf ) {
    // get current time
    let now = time::now();
    // generate TimeDuration with duration = starting time
    let starting_time = now.tm_hour as f64 + (now.tm_min as f64)/60.0;
    let start = TimeDuration{   year: now.tm_year + 1900,
                                month: now.tm_mon + 1,
                                day: now.tm_mday,
                                duration: starting_time};
    println!("Time Stamp: {:02}:{:02}", now.tm_hour, now.tm_min);
    // write to file
    write_time_to_file( start, &file_name);
}

/* delete last line from file
 * last line contains start time, delete it when writing duration for that day
 */
fn delete_start_in_file( file_name: &PathBuf) {
    // open file and read content to string
    let mut content = match read_file_to_string( &file_name) {
        Ok(string)=> string,
        Err(e)    => e,
    };

    // delete last line, has always length 17
    let length = content.len();
    content.truncate( length - 17);

    let mut storage = File::create(file_name).expect("Something..");

    // Write whole content to file
    storage.write( content.as_bytes()).expect("Deleting start time: Could not write back.");

}

/* Get TimeDuration from String
 */
fn get_td( content: &String ) -> TimeDuration {

    let split: Vec<&str> = content.split(',').collect();

    // TODO: add error handling if split is not a proper array
    TimeDuration { year: split[0].parse().unwrap(),
                            month: split[1].parse().unwrap(),
                            day: split[2].parse().unwrap(),
                            duration: split[3].parse().unwrap() }
}

/* Print curent content of file
 */
fn print_stats( file_name: &PathBuf ) {

    println!("Printing Stats:");
    println!("----------------------------");
    // read file content
    let content = match read_file_to_string( &file_name) {
        Ok(string)=> string,
        Err(e)    => e,
    }; 
    // convert lines to TimeDurations and print them
    for line in content.lines() {
        let td = get_td(&line.to_owned());
        println!("{}", td);
    }
    println!("----------------------------");
}

/* Calculate Statistics for months
 */
fn calc_stats( file_name: &PathBuf) {
    
    println!("Time per month:");
    // read file content to string
    let content = match read_file_to_string( file_name ){
        Ok(stor)    => stor,
        Err(err)    => format!("Open Error: {}", err),
    };
    // get all lines, last one is emtpy so we pop it
    let mut lines: Vec<&str> = content.split('\n').collect();
    lines.pop();
    // convert strings in each line to TimeDuration
    let line_td: Vec<TimeDuration> = lines.into_iter().map(|s| get_td( &s.to_string() )).collect();
    // get all months as Vector
    let months = get_months(&line_td);

    // sum up durations in the same month
    for month in months.iter() {
        let sum_dur: f64 = line_td.iter().filter(|x| x.month == *month).map(|x| x.duration).sum();
        //let sum_dur = month_td.iter().map(|x| x.month).sum();
        println!("Sum for {:>2}: {:>5.02}h",month, sum_dur);
    }
}

/* Read content from file
 * take file name and return content as String
 */
fn read_file_to_string( file_name: &PathBuf ) -> Result<String, String> {
    // open file, return error message
    let mut storage = match File::open(file_name) {
        Ok(stor)    => stor,
        Err(err)    => return Err( format!("Open Error: {}", err)),
    };
    // read file to string and return it, error returns empty string
    let mut content = String::new();
    match storage.read_to_string( &mut content) {
        Ok(_)  => Ok(content),
        Err(_) => {
            println!("Could not read content of file!");
            Err(String::from(""))
        },
    }

}

/* get months in Vector of TimeDurations
 */
fn get_months( line_td: &Vec<TimeDuration>) -> Vec<i32> {
    let mut months: Vec<i32> = Vec::new();
    // iterate of Vector and find those which are not in the months vector and add them
    for item in line_td.iter() {
        if months.iter().find(|x| x == &&item.month) == None {
            months.push(item.month);
        }
    }

    months
}