# ALPHA RELEASE - Cyclops - City of Heroes log parser
## I am open to suggestions for data you want to see. 


## Description
Application for parsing game chat logs into analysis reports.

## Preparation
Please, follow setup instructions otherwiase you will log incomplete or no data at all.

Under Option->Windows->Chat, Set "Log Chat" to Enabled, hit, Apply Now.
![window chat log settings](chat_log_settings.png)

In your chat Window, select the, Combat, tab, right click and edit.
Make sure these categories are in the left column:
- Combat Warnings
- Damage Inflicted
- Hit Rolls
- General Combat
- Damage Received
- Error
- Healing Received
- Healing Delivery
- Pet Damage Inflicted - Pet categories are very important or pseudo pet logging might be missed.
- Pet Hit Rolls
- Pet Damage Received
- Pets
- Pet Healing Received
- Pet Healing Delivery

![combat tab settings](combat_chat_settings.png)

Once, you log into a character, log files should start appearing in \<coh install dir\>/accounts/\<account name\>/Logs

## Cyclops instructions

Usage: cyclops.exe [OPTIONS]

  Options:  
  -l, --logdir \<Directories where your game chat files are stored. All files in the directory will be processed.\>  
  -f, --files \<List of game log files comma separated.\>  
  -i, --interval \<Time in seconds between combat sessions for DPS reports\>  
  -o, --outputdir \<Directory where you want the reports written. Defaults to "output"\>  
  -a, --address \<IP address the web server should use. Defaults to 127.0.0.1\>  
  -p, --port \<Port number the web server should use. Defaults to 11227\>  
  -h, --help Print help  
  -V, --version   

  You must have at least --files or --logdir on your command line. The rest are optional.  
    Examples:  
      cyclops --logdir d:\coh\accounts\fake\Logs  
      cyclops --logdir='d:\coh\accounts\fake\Logs','d:\coh\accounts\fake2\Logs'
      cyclops --files d:\coh\accounts\fake\Logs\'chatlog 2024-02-10.txt'  
      cyclops --files='d:\coh\accounts\fake\Logs\'chatlog 2024-02-10.txt','cyclops --files d:\coh\accounts\fake\Logs\'chatlog 2024-02-10.txt'  
      cyclops --interval=42 --files d:\coh\accounts\fake\Logs\'chatlog 2024-02-10.txt'  
      cyclops --logdir d:\coh\accounts\fake\Logs --outputdir e:\putfileshere  
      cyclops --logdir d:\coh\accounts\fake\Logs --outputdir e:\putfileshere -a 192.111.222.1 -p 8080


## Web server

  After log processing the application will start a HTTP web server on 127.0.0.1:11227 by default. Navigate to http://127.0.0.1:11227 to see an index page off all your processed log files. There will be four columns, date of the log file, player names that appear in the log, number of data points per summary, and the log file name.

  Click on the date of the log to go to the details page. The summary page has each play session separated by tabs for the selected log file.

  When you are done look at the data. Press Control-C in the command window to terminate the application. I plan to make this more user friendly in the future.

## Report Directory

  If you want at the files directly look in the, Report Directory, printed during the application run. Default location is the "output" directory where you ran the application. Example: Report directory: "D:\\cyclops\\output\\beta.data.staff.ice.stalker.1.29.txt"  

## Report directory structure:  
    Directory name is designed to limit the chance of you overwriting log files when the chat file name is the same from different accounts.  
      \<player name\>\_\<log file date\>
      Example: night_pixie_2024_02_08

## Inside the report directory  
      Summary of the report session.   
        - summary.html - Contains all the sessions in a log file as tabs.
        - What is a session? Each time you log in or out. Or use /local START PARSE or /local END PARSE a new session is assumed.  
        - You can have multiple characters in the same log. Or multiple sessions of the same character.  
        - Copy of the source chat log. Example: chatlog_2024_02_08.txt  
        - dps.csv - Raw dps data using for the dps report in CSV format.  
        - parsed.txt - Log files parsed into internal format. Useful for finding missed log messages. Look for, Unparsed.
        - summary.db - An Sqlite version 3.2+ database of all the data currently collected. Everything is tied together by the summary_key field in the table, Summary.

### Summary.html  
        - Attack Summary - Global combat totals for this session  
        - Attack Summary By Power - Combat totals per power  
        - DPS using an interval of \<interval\> - DPS (Damage per second) when the gap between damage log messages is less than the interval. Example, you attack a spawn, defeat them, wait 60 seconds, then attack another spawn. That would be considered two DPS sessions with an interval of 60.  







