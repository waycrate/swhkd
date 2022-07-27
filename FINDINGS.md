# Findings.

This is basically a summary of our findings whilst trying to become
totally independent of root access.

## Seat

One of our first suggestions where opening a seat to get access to the
keyboard and other input devices. After a lot of research we concluded
this path unviable. This is going to be technical.

So basically when a process asks the systems seat manager if they could
open a seat they indirectly ask to take control of the whole session.

Well in simple terms it means if we go ahead and implement seat support
the compositor/display server can't access a keyboard and mouse anymore
totally breaking normal expected compositor behaviour. Big nono.

## Pkexec

Pkexec is our current solution... (Somebody else do this i have no idea.)

## Polkit

Polkit is a solution we have considered... (Somebody else do this i have
no idea.)
