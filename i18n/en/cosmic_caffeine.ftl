caffeine-mode = Caffeine Mode
status-off = Caffeine is off
status-remaining = { $mode } · { $time } remaining
status-active = { $mode } mode active
tooltip-active = Active
tooltip-inactive = Inactive

timer-infinity = Infinity
timer-five-mins = 5 Minutes
timer-ten-mins = 10 Minutes
timer-thirty-mins = 30 Minutes
timer-one-hour = 1 Hour
timer-two-hours = 2 Hours
timer-three-hours = 3 Hours
timer-four-hours = 4 Hours
timer-manual = Custom
timer-hours-label = Hours
timer-minutes-label = Minutes

inhibit-mode = Inhibit Mode
inhibit-idle = Idle
inhibit-suspend = Suspend
inhibit-both = Both
start-caffeine = Enable
stop-caffeine = Disable

time-hours-minutes = { $hours }h { $mins }m
time-minutes = { $mins }m
time-seconds = { $secs }s

notification-enabled-title = Caffeine Mode Activated
notification-enabled-body = Caffeine is active until you turn it off.
notification-enabled-timed-body = { $mins ->
    [one] Caffeine is active for one minute.
   *[other] Caffeine is active for { $mins } minutes.
    }
notification-disabled-title = Caffeine Mode Deactivated
notification-disabled-body = Normal power settings restored.
notification-timer-expired-title = Caffeine Timer Expired
notification-timer-expired-body = Normal power settings have been restored.
notification-warning-title = Caffeine Timer Warning
notification-warning-body = { $mins ->
    [one] One minute remaining.
   *[other] { $mins } minutes remaining.
    }
notification-error-title = Caffeine Error
notification-error-body = Could not turn on Caffeine.

env-warning = This applet is designed for COSMIC and may work differently on other desktops.
