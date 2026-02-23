Scriptname DialogueFollowerScript extends Quest
{Modified vanilla script to support multiple followers while maintaining vanilla features like animals.}

; -- Custom Properties --

MultiFollowerManager property Manager auto
{Reference to our new manager quest script.}

; -- Vanilla Properties (Keep names exact for CK compatibility) --

GlobalVariable Property PlayerFollowerCount Auto
GlobalVariable Property PlayerAnimalCount Auto

Faction Property CurrentFollowerFaction Auto
Faction Property CurrentAnimalFaction Auto

Message Property FollowerDismissMessage Auto
Message Property FollowerDismissMessageWedding Auto
Message Property FollowerDismissMessageCompanions Auto
Message Property FollowerDismissMessageCompanionsExtra Auto
Message Property FollowerDismissMessageWait Auto
Message Property AnimalDismissMessage Auto
Message Property AnimalDismissMessageWait Auto

ReferenceAlias Property Follower Alias
ReferenceAlias Property Animal Alias

; -- Follower Functions --

Function SetFollower(ObjectReference FollowerRef)
    Actor followerActor = FollowerRef as Actor
    if (followerActor)
        if (Manager.AddFollower(followerActor))
            followerActor.SetFactionRank(CurrentFollowerFaction, 0)
            ; We don't fill the vanilla 'Follower' alias here because our
            ; Manager handles its own 5 aliases.
        else
            Debug.Notification("You cannot take any more followers.")
        endif
    endif
EndFunction

Function DismissFollower(Int iMessageMode, Int iDismissMode)
    Actor target = Game.GetCurrentDialogueTarget() as Actor

    ; If we aren't talking to anyone, try the vanilla alias as a fallback
    if (!target)
        target = Follower.GetActorReference()
    endif

    if (target && Manager.IsFollower(target))
        Manager.RemoveFollower(target)
        target.SetFactionRank(CurrentFollowerFaction, -1)

        if iMessageMode == 0
            FollowerDismissMessage.Show()
        elseif iMessageMode == 1
            FollowerDismissMessageWedding.Show()
        elseif iMessageMode == 2
            FollowerDismissMessageCompanions.Show()
        endif
    endif
EndFunction

; -- Animal Functions (Restored Vanilla Logic) --

Function SetAnimal(ObjectReference AnimalRef)
    Actor animalActor = AnimalRef as Actor
    if (animalActor)
        Animal.ForceRefTo(animalActor)
        animalActor.SetFactionRank(CurrentAnimalFaction, 0)
        animalActor.SetPlayerTeammate(true)
        PlayerAnimalCount.SetValue(1)
    endif
EndFunction

Function DismissAnimal(Int iMessageMode, Int iDismissMode)
    Actor animalActor = Animal.GetActorReference()
    if (animalActor)
        Animal.Clear()
        animalActor.SetFactionRank(CurrentAnimalFaction, -1)
        animalActor.SetPlayerTeammate(false)
        PlayerAnimalCount.SetValue(0)

        if iMessageMode == 0
            AnimalDismissMessage.Show()
        endif
    endif
EndFunction
