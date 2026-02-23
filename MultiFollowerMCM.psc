Scriptname MultiFollowerMCM extends SKI_ConfigBase
{MCM for managing multiple followers.}

; -- Properties --

MultiFollowerManager property Manager auto
{Reference to the manager script.}

; -- Variables --

int[] followerOIDs
int teleportAllOID
int waitAllOID
int followAllOID

; -- Events --

Event OnConfigInit()
    Pages = new string[2]
    Pages[0] = "Current Followers"
    Pages[1] = "Global Commands"

    followerOIDs = new int[5]
EndEvent

Event OnPageDraw(string page)
    if page == "Current Followers"
        DrawFollowerPage()
    elseif page == "Global Commands"
        DrawGlobalPage()
    endif
EndEvent

Event OnSelectST(string stateName)
    ; Individual commands could be handled here if we use states
EndEvent

Event OnDefaultST(string stateName)
EndEvent

Event OnHighlightST(string stateName)
EndEvent

; -- Page Drawing --

function DrawFollowerPage()
    SetCursorFillMode(TOP_TO_BOTTOM)

    int i = 0
    while (i < 5)
        Actor f = Manager.Followers[i]
        if (f != None)
            string name = f.GetActorBase().GetName()
            string status = "Following"
            if (f.GetActorValue("WaitingForPlayer") == 1)
                status = "Waiting"
            endif

            AddHeaderOption(name + " (" + status + ")")
            followerOIDs[i] = AddTextOption("Dismiss", "Click to Dismiss")
        else
            AddHeaderOption("Slot " + (i + 1) + ": [Empty]")
            followerOIDs[i] = -1
        endif
        i += 1
    endWhile
endFunction

function DrawGlobalPage()
    SetCursorFillMode(TOP_TO_BOTTOM)

    teleportAllOID = AddTextOption("Teleport All", "Click to execute")
    waitAllOID = AddTextOption("Wait All", "Click to execute")
    followAllOID = AddTextOption("Follow All", "Click to execute")
endFunction

; -- Event Handling --

Event OnSelect(int option)
    ; Global Commands
    if (option == teleportAllOID)
        Manager.TeleportAll()
        ShowMessage("All followers teleported.", false)
    elseif (option == waitAllOID)
        Manager.WaitAll()
        ForcePageReset()
    elseif (option == followAllOID)
        Manager.FollowAll()
        ForcePageReset()
    else
        ; Check if it's a dismiss button
        int i = 0
        while (i < 5)
            if (option == followerOIDs[i])
                Actor f = Manager.Followers[i]
                if (f != None)
                    string name = f.GetActorBase().GetName()
                    if (Manager.RemoveFollowerByIndex(i))
                        ShowMessage(name + " has been dismissed.", false)
                        ForcePageReset()
                    endif
                endif
            endif
            i += 1
        endWhile
    endif
EndEvent
