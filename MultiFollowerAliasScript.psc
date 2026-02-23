Scriptname MultiFollowerAliasScript extends ReferenceAlias
{Script attached to each follower alias to handle death and cleanup.}

MultiFollowerManager property Manager auto

Event OnDeath(Actor akKiller)
    Actor selfActor = GetActorReference()
    if (selfActor)
        Debug.Notification(selfActor.GetActorBase().GetName() + " has died.")
        Manager.RemoveFollower(selfActor)
    endif
EndEvent

Event OnUnload()
    ; Optional: handle cleanup if they are unloaded for too long?
    ; Better to stick to OnDeath for now.
EndEvent
