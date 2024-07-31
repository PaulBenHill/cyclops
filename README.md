# BETA RELEASE - Cyclops 1.1 - City of Heroes log parser
## I am open to suggestions for data you want to see. 


## Description
Application for parsing game chat logs into analysis reports.

## Preparation
Please, follow setup instructions otherwiase you will log no or incomplete data. You will have to this once for any character you want to parse.

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
- Pet Combat
- Pet Damage Inflicted - Pet categories are very important or pseudo pet logging might be missed.
- Pet Hit Rolls
- Pet Damage Received
- Pets
- Pet Healing Received
- Pet Healing Delivery

![combat tab settings](combat_chat_settings.png)

At this point, I strongly suggest you log back to the character selection screen. It will make sure a clean session start occurs after all the changes.

Once, you log into a character, log files should start appearing in \<coh install dir\>/accounts/\<account name\>/Logs

## Cyclops instructions

The default behavior is to start the web server at http://127.0.0.1:11227. Command line options are available. See below.

## Windows Users: 
- double click on - cyclops.bat
## Linux/IOS: 
- I can provide a binary, but it's untested for now
- Open terminal window
- cd to Cyclops install directory
- ./cyclops.exe

## Web server [http://127.0.0.1:11227](http://127.0.0.1:11227)

  The application will start a HTTP web server on http://127.0.0.1:11227 by default. Navigate to http://127.0.0.1:11227 to see an index page off all your processed log files. There will be four columns, date of the log file, player names that appear in the log, number of data points per summary, and the log file name. See below: Index.html

  Click on the player name to see that character's summary. The summary page has each play session separated by tabs for the selected log file. See below: Summary.html

  When you are done using the tool. Press Control-C in the command window to terminate the application. Or close the command box. I plan to make this more user friendly in the future.

## Report Directory

  If you want at the files directly look in the, Report Directory, printed during the application run. Default location is the "output" directory where you ran the application. Example: Report directory: "D:\\cyclops\\output\\beta.data.staff.ice.stalker.1.29"  

    
### Index.html
- Default landing page of the web server. Default: http://127.0.0.1:11227
- Lists all summaries for each session. You can have multiple characters in the same log. Or multiple sessions of the same character.  
  - What is a session?
    - A session covers the log time period between each login.
    - Or the time period between each use of /local STARTPARSE $name or /local ENDPARSE $name
      - STARTPARSE is mostly used for repeative tests like Pylons.
    - Actions:
      - The drop down lists contains any directory you have previous uploaded.
      - Parse Directory - Parse every file in the select directory.
      - Parse Newest File In Directory - Parse the last file modified. Usually this is the last played session.
      - Text field is for copying the full path of the file or directory for uploading. You MUST provide the full path.
        - Use Control+Shift+C in the File Explorer to get the full path.
      -  Parse File - Parse a single file.
      - Parse Directory - Parse all files in a directory. Afterwards, the directory will appear in the drop down list.
  - Search
    - List only summaries for a selected player
    - List only summaries from a selected directory
    - Reload summary table contents
        
### Summary.html 
- Attack Summary - Global totals for this session
- Attack Summary By Power - Combat totals per power
  - Merge rows together that you think are related. Like procs.
  - Delete row
- Remove Non Damage Power - Removes powers like Hasten, Placate, and Build Up.
- Revert Changes - Revert all table changes.
- DPS using an interval of \<interval\> - DPS (Damage per second) when the gap between damage log messages is less than the interval. Example, you attack a spawn, defeat them, wait 60 seconds, then attack another spawn. That would be considered two DPS sessions with an interval of 60.
- Damage Dealt By Type - Damage done to mobs sorted by damage type.
- Damage Taken By Type - Damage dealt to the player by damage type.
- Damage Taken By Mob - General summary of damage dealt to the player by each mob.
- Damage Taken By Mob Power - Detailed break down of damage dealt to the player by each mob's power.
- Damage Dealt To Mob Power - Detailed break down of damage dealty by each player power for each mob damaged.

## Report directory is where the data is stored to generate the summaries
- Copy of the source chat log. Example: chatlog_2024_02_08.txt.
- Copy of each session broken out as a separate file.
  - \<0 indexed session id\>\_player\_\<starting line in the full chat log>.txt.
    - Example: 0_Elena_Taiga_20.txt
  - Used for double checking numbers
  - I would be forever grateful if you doubled checked numbers that looked off.
- dps.csv - Raw dps data using for the dps report in CSV format.  
- parsed.txt - Log files parsed into internal format. Useful for finding missed log messages. Look for, Unparsed.
- summary.db - An Sqlite version 3.2+ database of all the data currently collected. Everything is tied together by the summary_key field in the table, Summary.

## Command line options if you want to change defaults to parse things outside the UI
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

  Everything below is optional.
    Examples:  
      cyclops --logdir d:\coh\accounts\fake\Logs  
      cyclops --logdir='d:\coh\accounts\fake\Logs','d:\coh\accounts\fake2\Logs'
      cyclops --files d:\coh\accounts\fake\Logs\'chatlog 2024-02-10.txt'  
      cyclops --files='d:\coh\accounts\fake\Logs\'chatlog 2024-02-10.txt','cyclops --files d:\coh\accounts\fake\Logs\'chatlog 2024-02-10.txt'  
      cyclops --interval=42 --files d:\coh\accounts\fake\Logs\'chatlog 2024-02-10.txt'  
      cyclops --logdir d:\coh\accounts\fake\Logs --outputdir e:\putfileshere  
      cyclops --logdir d:\coh\accounts\fake\Logs --outputdir e:\putfileshere -a 192.111.222.1 -p 8080

### Data Notes
- I round all numbers to the nearest whole number. Using rounding functions, not truncation. This does introduce small difference between the tool values and what you would get if you added the log values up manually. Around +/- 2%.
- If you want to investigate the database directly. A good tool is [Sqlite Studio](https://www.sqlitestudio.pl/)

### Super geeky technical stuff
- [Github repo](https://github.com/PaulBenHill/cyclops)
- [Rust language for the back end](https://www.rust-lang.org/)
- [HTMX JS library for the UI](https://htmx.org/)
- [Tera for templating](https://keats.github.io/tera/)
- [Actix for the web server](https://actix.rs/)
- [Sqlite for the database](https://www.sqlite.org/)







