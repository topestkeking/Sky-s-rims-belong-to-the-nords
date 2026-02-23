Scriptname MultiFollowerManager extends Quest
{Core script to manage multiple followers.}

; -- Properties --

ReferenceAlias[] property FollowerAliases auto
{Array of 5 aliases from the DialogueFollower quest.}

Actor[] property Followers auto
{Array to hold the 5 followers.}

GlobalVariable property PlayerFollowerCount auto
{Vanilla global tracking follower count.}

Faction property PlayerFollowerFaction auto
{The faction that makes them a follower.}

Message property DismissMessage auto
{Message shown when a follower is dismissed.}

; -- Variables --

int MaxFollowers = 5

; -- Initialization --

Event OnInit()
    Followers = new Actor[5]
EndEvent

; -- Public Functions --

bool function AddFollower(Actor akActor)
    if (IsFollower(akActor))
        return false
    endif

    int index = GetEmptySlot()
    if (index == -1)
        Debug.Notification("You already have too many followers.")
        return false
    endif

    Followers[index] = akActor

    ; Fill alias if available
    if (FollowerAliases && index < FollowerAliases.Length && FollowerAliases[index] != None)
        FollowerAliases[index].ForceRefTo(akActor)
    endif

    akActor.SetFactionRank(PlayerFollowerFaction, 0)
    akActor.SetPlayerTeammate(true)
    akActor.SetActorValue("WaitingForPlayer", 0)

    UpdateVanillaGlobal()
    return true
endFunction

bool function RemoveFollower(Actor akActor)
    int index = FindFollowerIndex(akActor)
    if (index != -1)
        return RemoveFollowerByIndex(index)
    endif
    return false
endFunction

bool function RemoveFollowerByIndex(int index)
    if (index < 0 || index >= MaxFollowers)
        return false
    endif

    Actor followerActor = Followers[index]
    if (followerActor == None)
        return false
    endif

    Followers[index] = None

    ; Clear alias if available
    if (FollowerAliases && index < FollowerAliases.Length && FollowerAliases[index] != None)
        FollowerAliases[index].Clear()
    endif

    followerActor.SetFactionRank(PlayerFollowerFaction, -1)
    followerActor.SetPlayerTeammate(false)
    followerActor.SetActorValue("WaitingForPlayer", 0)

    UpdateVanillaGlobal()
    return true
endFunction

bool function IsFollower(Actor akActor)
    return FindFollowerIndex(akActor) != -1
endFunction

int function GetFollowerCount()
    int count = 0
    int i = 0
    while (i < MaxFollowers)
        if (Followers[i] != None)
            count += 1
        endif
        i += 1
    endWhile
    return count
endFunction

function WaitAll()
    int i = 0
    while (i < MaxFollowers)
        if (Followers[i] != None)
            Followers[i].SetActorValue("WaitingForPlayer", 1)
        endif
        i += 1
    endWhile
    Debug.Notification("Your followers are waiting.")
endFunction

function FollowAll()
    int i = 0
    while (i < MaxFollowers)
        if (Followers[i] != None)
            Followers[i].SetActorValue("WaitingForPlayer", 0)
        endif
        i += 1
    endWhile
    Debug.Notification("Your followers are following.")
endFunction

function TeleportAll()
    Actor player = Game.GetPlayer()
    int i = 0
    while (i < MaxFollowers)
        if (Followers[i] != None)
            Followers[i].MoveTo(player)
        endif
        i += 1
    endWhile
endFunction

; -- Internal Functions --

int function GetEmptySlot()
    int i = 0
    while (i < MaxFollowers)
        if (Followers[i] == None)
            return i
        endif
        i += 1
    endWhile
    return -1
endFunction

int function FindFollowerIndex(Actor akActor)
    int i = 0
    while (i < MaxFollowers)
        if (Followers[i] == akActor)
            return i
        endif
        i += 1
    endWhile
    return -1
endFunction

function UpdateVanillaGlobal()
    int count = GetFollowerCount()
    if (count > 0)
        PlayerFollowerCount.SetValue(1)
    else
        PlayerFollowerCount.SetValue(0)
    endif
endFunction
