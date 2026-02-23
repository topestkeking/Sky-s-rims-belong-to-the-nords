# Multiple Follower Mod - Implementation Guide

This guide explains how to set up the Multiple Follower system in the Skyrim Creation Kit using the provided scripts.

## Step 1: Install Scripts
1. Copy the following files into your `Data/Scripts/Source` folder:
   - `MultiFollowerManager.psc`
   - `MultiFollowerAliasScript.psc`
   - `DialogueFollowerScript.psc` (Note: This replaces the vanilla source, back up the original if needed)
   - `MultiFollowerMCM.psc`
2. Compile these scripts in the Creation Kit or using your preferred Papyrus compiler.

## Step 2: Set up the Manager Quest
1. In the Creation Kit, create a new **Quest**.
   - **ID:** `MultiFollowerManagerQuest`
   - **Name:** `Multi Follower Manager`
   - **Start Game Enabled:** Checked.
2. Go to the **Scripts** tab and add `MultiFollowerManager.psc`.
3. Set the properties for `MultiFollowerManager`:
   - `PlayerFollowerCount`: Link to the global variable `PlayerFollowerCount`.
   - `PlayerFollowerFaction`: Link to the faction `CurrentFollowerFaction`.
   - `FollowerAliases`: Link this to the 5 aliases you create in the `DialogueFollower` quest (see Step 3).
   - `Followers`: This will be managed by the script at runtime.

## Step 3: Modify the Vanilla `DialogueFollower` Quest
1. Open the vanilla quest `DialogueFollower`.
2. Go to the **Scripts** tab.
3. You will see the existing `DialogueFollowerScript`.
4. **IMPORTANT:** You should ideally replace it with the one provided or update the existing one with the logic I provided.
5. Set the new `Manager` property on this script to point to your `MultiFollowerManagerQuest`.

### Adding Extra Aliases (Highly Recommended)
In Skyrim, followers usually move because of an **AI Package** attached to the `Follower` alias in the `DialogueFollower` quest. To make multiple followers work correctly:
1. Go to the **Quest Aliases** tab in the `DialogueFollower` quest.
2. You will see an alias named `Follower`.
3. Create 4 more aliases: `Follower2`, `Follower3`, `Follower4`, and `Follower5`.
4. For each new alias, copy the exact settings from the original `Follower` alias (including the **AI Packages**).
5. Ensure the aliases are set to "Optional".
6. **Attach the Script:** To each of the 5 aliases (including the original), add the `MultiFollowerAliasScript.psc` and link the `Manager` property to your `MultiFollowerManagerQuest`.

## Step 4: Fix Recruitment Dialogue (Crucial!)
By default, Skyrim hides the "Follow me" dialogue if you already have one follower. You must fix this in the Creation Kit:
1. Open the **Quest** `DialogueFollower`.
2. Go to the **Player Dialogue** tab.
3. Look for the branches related to recruitment (e.g., `DialogueFollowerRecruitment`).
4. In the **Conditions** list for these lines, you will see: `GetGlobalValue PlayerFollowerCount == 0.00`.
5. **Change this condition:** Set it to `GetGlobalValue PlayerFollowerCount <= 1.00` OR simply **delete** the condition.
   - *Note: Our script will manually show a "You cannot take any more followers" message if you try to recruit a 6th person.*

## Step 5: Set up the MCM (SkyUI)
1. Create a new **Quest**.
   - **ID:** `MultiFollowerMCMQuest`
   - **Name:** `Multi Follower MCM`
2. Go to the **Scripts** tab and add `MultiFollowerMCM.psc`.
3. Set the properties:
   - `Manager`: Link to your `MultiFollowerManagerQuest`.
4. Ensure this quest is also "Start Game Enabled".

## How to Use In-Game
1. Recruit followers using the standard "Follow me" dialogue. You can now have up to 5.
2. Open the **Mod Configuration Menu** (MCM) in the escape menu to:
   - See who is currently following you.
   - Dismiss them remotely.
   - Use "Mass Commands" like Teleport All, Wait All, or Follow All.

## Compatibility Notes
- This mod modifies the vanilla `DialogueFollowerScript`. It will conflict with other mods that do the same (like AFT, EFF, or UFO).
- It is designed to be a lightweight alternative to those heavy frameworks.
