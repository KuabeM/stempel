## Working Time Calculator

Program to calculate the current working time per month/week and so on.

Project folder: `/working-time-calc/`

Usage:

    working-time-calc start | stop | show | version

- `start` writes current time to text file `time_storage.txt` (hard-coded) in the same directory
- `stop` reads last line in `time_storage.txt`, calculates the elapsed time till now and writes that value to last line instead of starting time
- `show` prints all data sets from file and the summed-up time per month to console
- `version` prints current version of programm 
