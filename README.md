# ALPHA RELEASE - Cyclops - City of Heroes log parser
## I am open to suggestions for data you want to see. 
## All the HTML style is embedded in ./templates/player_attack_report.html. Feel free to suggest better styling.


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

Once, you log into a character, log files should start appearing in <coh install dir>/accounts/<account name>/Logs

## Cyclops instructions
Usage: cyclops.exe [OPTIONS]

Options:
  -l, --logdir <Directory where you game chat files are stored. All files in the directory will be processed.>
  -f, --files <List of game log files comma separated.>
  -i, --interval <Time in seconds between combat sessions for DPS reports>
  -o, --outputdir <Directory where you want the reports written. Defaults to "output">
  -h, --help                                                                                                    Print help
  -V, --version 

You must have at least --files or --logdir on your command line. The rest are optional.
Examples:
cyclops --logdir d:\coh\accounts\fake\Logs
cyclops --files d:\coh\accounts\fake\Logs\'chatlog 2024-02-10.txt'
cyclops --interval=42 --files d:\coh\accounts\fake\Logs\'chatlog 2024-02-10.txt'
cyclops --logdir d:\coh\accounts\fake\Logs --outputdir e:\putfileshere


The look in the, Report Directory, printed during the application run. Default location is the "output" directory where you ran the application. Example: Report directory: "D:\\cyclops\\output\\beta.data.staff.ice.stalker.1.29.txt_290681"

Report directory structure:
Directory name is designed to limit the chance of you overwriting log files when the chat file name is the same from different accounts.
    <chat log file name>_<file size in bytes>
    Example: chatlog_2024_02_08.txt_1260281
Inside the report directory:
Summary of the report session. 
    <player name>_<session number>_summary.html
    Each time you log in or out. Or use /local START PARSE or /local END PARSE a new session is assumed.
    You can have multiple characters in the same log. Or multiple sessions of the same character.
Copy of the source chat log. Example: chatlog_2024_02_08.txt
dps.csv - Raw dps data using for the dps report in CSV format.
effected_report.csv - Targets effected data in csv format.
parsed.json - Log files parsed into JSON format.
parsed.txt - Log files parsed into internal format. Useful for finding missed log messages. Look for, Unparsed.

Summary.html
Attack Summary - Global combat totals for this session
Attack Summary By Power - Combat totals per power
DPS using an interval of <interval> - DPS (Damage per second) when the gap between damage log messages is less than the interval. Example, you attack a spawn, defeat them, wait 20 seconds, then attack another spawn. That would be considered two DPS sessions with an interval of 20.
Number of targets effected by power (experimental) - How many targets were effected by an power even if the attack missed. A rough idea of how many mobs a player is hitting in gameplay with a power.







