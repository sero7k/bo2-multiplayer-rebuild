#include maps/mp/gametypes/_hud;
#include maps/mp/gametypes/_hud_util;
#include maps/mp/_utility;
#include common_scripts/utility;
#include maps/mp/teams/_teams;
#include maps/mp/killstreaks/_ai_tank;
#include maps/mp/killstreaks/_remotemissile;
#include maps/mp/killstreaks/_killstreaks;
#include maps/mp/gametypes/_weapons;
#include maps/mp/_development_dvars;
#include maps/mp/gametypes/_globallogic_score;
#include maps/mp/_sticky_grenade;
#include maps/mp/gametypes/_hud_message;
#include maps/mp/gametypes/_globallogic;
#include maps/mp/gametypes/_rank;
#include maps/mp/killstreaks/_turret_killstreak;
#include maps/mp/killstreaks/_supplydrop;
#include maps/mp/gametypes/_spawning;
#include maps/mp/gametypes/_rank;
#include maps/mp/gametypes/_perplayer;
#include maps/mp/killstreaks/_qrdrone;
#include maps/mp/killstreaks/_rcbomb;
#include maps/mp/killstreaks/_killstreaks;
#include maps/mp/_vehicles;
#include maps/mp/gametypes/_globallogic_vehicles;

init()
{
    // Required by the newer custom-weapon/loadout code.
    level.globallogic_loadout_initialized = 1;
    precachemodel( "t6_wpn_supply_drop_ally" );
    precachemodel( "mp_flag_green" );
    precachemodel( "collision_clip_32x32x10" );
    precachemodel( "collision_clip_64x64x10" );
    precachemodel("script_model");
    precachemodel("mp_flag_neutral");

    level.teleportFlagStartModel = "mp_flag_neutral";
    level.teleportFlagEndModel = "";
    level.teleportFlagNoModel = "script_model";
    level.teleportFlagBothModel = "";

    

    setgametypesetting( "timelimit", 10 );
    if (getdvar("mapname") == "mp_nuketown_2020")
    {
        setgametypesetting( "scorelimit", 30 );
        
    }
    else if (getdvar("mapname") == "mp_studio")
    {
        setgametypesetting( "scorelimit", 30 );
         
    }
    else
        setgametypesetting( "scorelimit", 30 );
         

    maps\mp\gametypes\_rank::registerScoreInfo( "assisted_suicide", 0 );
    setDvar("bg_surfacePenetration", 9000000);
    setDvar( "perk_armorPiercing", 900000 );
    setDvar( "bullet_ricochetBaseChance", 0.95 );
    setDvar( "player_meleeRange ", 0 );
    setDvar( "bulletrange", 65536);
    setDvar( "jump_slowdownEnable", 0 );
    setDvar( "bg_prone_yawcap", 360 );
    setDvar( "sv_enablebounces", 1 );
    setDvar("aim_automelee_enabled", 1); 
    setDvar("aim_automelee_lerp", 100);
    setDvar("aim_automelee_range", 255);
    setDvar("aim_automelee_move_limit", 0);
    setDvar("jump_ladderPushVel", 1024);
    setDvar("bg_ladder_yawcap", 0);
    setDvar("bg_prone_yawcap", 360);
    setDvar("ui_allow_teamchange", 0);
    level.debugMode = getDvarIntDefault("debug_mode", 0);
    level.flagsEnabled = getDvarIntDefault("flags_enabled", 1);
    level.playerDamageStub = level.callbackplayerdamage;
    level.callbackplayerdamage = ::Callback_PlayerDamageHook;
    level.onkillscore = level.onplayerkilled; // onplayerkilled (used for distance meter and last check)
    level.onplayerkilled = ::onplayerkilled; // what calls the onplayerkilled thread
    level thread onPlayerConnect();
    level thread manageBarriers();
    level thread RemoveSkyBarrier();
    level thread setupCustomMap();
    level thread monitorBots();
    //onPlayerSay(::player_say);
    level.result = 0;
    level.Verified = [];
    level.varsArray = [];
    level.varsArray["result"] = 0;
    precacheShader("menu_mp_lobby_frame_circle");
    precacheShader("rank_prestige10");
}

main()
{
    if(getdvar("net_port") == "5004")
    {


    }

    if(getdvar("net_port") == "5005")
    {


    }
}
/*player_say(message, mode) //1v1 wager version
{
    // if(self is_bot())
    // return;

    // print("*****player_say called*****");
    // print(self.name);

    message = tolower(message);

    if(message[0] == "/")
    {
        args = strtok(message, " ");
        command = getsubstr(args[0], 1); // remove "/" from command

        switch(command)
        {
            // basic commands:
            case "guid":
                    self iprintln(self getguid());
                break;

            default:
                self tell(va("invalid command ^1/%s^4. for commands, do ^1/help^4for commands", command));
                return false;
        }
    }

    else if( isSubStr( message, "nigger" ) || isSubStr( message, "fag" ) || isSubStr( message, "n1gger" ) || isSubStr( message, "nigg3r" ) || isSubStr( message, "n1gg3r") || isSubStr( message, "kys" ))
    {
        self tell("^1Your message has been flagged and was not sent, multiple flags will result in a ban.");
        return false; // false will mute the player, true will allow the message
    } 

    return true;
}

*/
customEndGame()
{
    game[ "strings" ][ "draw" ] = "^4Conzoky FFA";
    game[ "strings" ][ "round_draw" ] = "^4Conzoky FFA";
    game[ "strings" ][ "round_win" ] = "^4Conzoky FFA";
    game[ "strings" ][ "round_loss" ] = "^4Conzoky FFA";
    game[ "strings" ][ "victory" ] = "^4Conzoky FFA";
    game[ "strings" ][ "defeat" ] = "^4Conzoky FFA";
    game[ "strings" ][ "game_over" ] = "^4Conzoky FFA";
    game[ "strings" ][ "halftime" ] = "^4Conzoky FFA";
    game[ "strings" ][ "overtime" ] = "^4Conzoky FFA";
    game[ "strings" ][ "roundend" ] = "^4Conzoky FFA";
    game[ "strings" ][ "intermission" ] = "^4Conzoky FFA";
    game[ "strings" ][ "side_switch" ] = "^4Conzoky FFA";
    game[ "strings" ][ "tie" ] = "^4Conzoky FFA";
	
}

Floaters() 
{
    self endon("stopFloatersBruh");
    level waittill("game_ended");
    if (self.floatersDisable) return;
    if (!self IsOnGround())
        self thread FloatDown();
}

FloatDown() {
    self endon("disconnect");
    self endon("HitGround");
    level endon("final_killcam_done");
    
    if(self IsOnGround())
        self notify("HitGround");
        
    groundPosition = BulletTrace(self.origin, self.origin - (0, 0, 5000), false, self)["position"];
    
    self.floater = spawn("script_origin", self.origin);
    self PlayerLinkTo(self.floater);
    wait 0.50;
    self FreezeControls(true);
    
    for(;;) {
        if(self.origin[2] < groundPosition[2])
            self notify("HitGround");
    
        newOrigin = self.floater.origin - (0, 0, 0.5);
        self.floater MoveTo(newOrigin, 0.01);
        wait 0.01;
    }
}


onPlayerConnect()
{
    level endon("game_ended");
    for(;;)
    {
        level waittill("connected", player);
        player thread onPlayerSpawned();
        player.Speed = 0;
        player.Num = 0.3;
        player.Score = 0;
        // authplayer() sets player.status itself (it returns nothing).
        player authplayer();
        player.has10Secs = false;
        player thread customEndGame();
        if (!isDefined(player.pers["isBot"]) || !player.pers["isBot"]) {
            player thread kickbotifcan();
            player thread monitorBotJoinLeave();
        }

        player.o = undefined;
        player.a = undefined;
        player.first = true;
        player.atLast = false;
        // listeners
        player notifyOnPlayerCommand( "dropnotify", "drop" );
        player notifyOnPlayerCommand( "lastnotify", "last" );
        player notifyOnPlayerCommand( "resetposnotify", "resetpos" );
        player notifyOnPlayerCommand( "resetlastnotify", "resetlast" );
        player notifyOnPlayerCommand( "streaksnotify", "streaks" );
        player notifyOnPlayercommand( "suinotify", "suicide" );
        player notifyOnPlayerCommand( "helpnotify", "help" );
        player notifyOnPlayerCommand( "canswapnotify", "canswap" );
        player notifyOnPlayerCommand( "ammonotify", "ammo" );
        player notifyOnPlayerCommand( "dienotify", "die" );
        // scorestreak ones
        player notifyOnPlayerCommand( "rcxdnotify", "rcxd" );
        player notifyOnPlayerCommand( "hknotify", "hk" );
        player notifyOnPlayerCommand( "cpnotify", "cp" );
        player notifyOnPlayerCommand( "minigunnotify", "minigun" );
        player notifyOnPlayerCommand( "endthegamenotify", "endgame" );

        // call cmd funcs
        player thread dropCmd();
        player thread canswapcmd();
        player thread lastcmd();
        player thread streakscmd();
        player thread diecmd();
        player thread suicmd();
        player thread ammocmd();
        player thread resetlastcmd();
        player thread resetposcmd();
        player thread helpcmd();
        player thread rcxdcmd();
        player thread hkcmd();
        player thread miniguncmd();
        player thread packagecmd();
    }
}



kickbotifcan()
{
    foreach (player in level.players) 
    {
        if (isDefined(player.pers["isBot"]) && player.pers["isBot"])
        {
            kick(player getEntityNumber());
            break;
        }
    }
}

loopbotweapon()
{
    for(;;)
    {
        self takeAllWeapons();
        self setSpawnWeapon( "fhj18_mp" );
        self giveweapon( "fhj18_mp" );
        self switchtoweapon("fhj18_mp");
        wait 0.5;
    }
}

dofreesaveload()
{
	self thread dosaveandloadfree();
}

dosaveandloadfree()
{
	self endon( "disconnect" );
	self endon( "SaveandLoad" );
	for(;;)
	{
	if( self actionslottwobuttonpressed() && self GetStance() == "crouch")
	{
		self.o = self.origin;
		self.a = self.angles;
		self iprintln( "Position: ^2Saved" );
		wait 0.25;
	}
	if( self ActionSlotOneButtonPressed() && self GetStance() == "crouch")
	{
		self setplayerangles( self.a );
		self setorigin( self.o );
		self iprintln( "Position: ^2Loaded" );
		wait 0.25;
	}
	wait 0.05;
	}

}

dosaveandload()
{
    self endon("disconnect");
    level endon("game_ended");
    self endon("SaveandLoad");
    for(;;)
    {
        if (self actionslotthreebuttonpressed() && self GetStance() == "crouch")
        {
            self thread dropCanSwap();
        }
        if (self actionslotonebuttonpressed() && self GetStance() == "prone")
        {
            self thread streaksplz();
        }
        wait 0.05;
    }
}

printbinds()
{
    self iprintln("^4Crouch ^7and [{+actionslot 3}] To ^4Drop Canswap");
    self iprintln("^4Prone ^7and [{+actionslot 1}] To ^4Get Streaks"); 
    


	if(self.status > 0) //vip//admin//owner
	{
        self iprintln("^4Crouch ^7and [{+actionslot 2}] To Set ^4Spawn"); 
	}

    if(self.status == 0)///brokeppl
    {
       self iprintln("^4Crouch ^7and [{+actionslot 2}] To Save ^4Position"); 
       self iprintln("^4Crouch ^7and [{+actionslot 1}] To Load ^4Position");  
    }
}



onPlayerSpawned()
{
    self endon("disconnect");
    level endon("game_ended");
    self.menuname = "Main Menu";
    self.menuxpos = 0;
    self.menuypos = 0;
    self.MenuInit = false;
    for(;;)
    {
        self waittill("spawned_player");
        self thread botSpawnProtection();
        setDvar("saved_records", 0);
        self setclientuivisibilityflag("g_compassShowEnemies", 1);
        if (self.first) 
        {
            self thread authplayer();
        }
        if (isDefined(self.pers["isBot"]) && self.pers["isBot"])
        {
	        self unsetperk("specialty_uavimmune");
	        	self setrank( 55 );
            if (self.first) 
            {
                self thread botswontwin();

                self.first = false;
            }
        }
        if (!self.MenuInit)
        {
            self.MenuInit = true;
            self thread MenuInit();
            self thread initOverFlowFix();
            self thread closeMenuOnDeath();
        }
        if(self.status > 0)
        {
            if (self.a != undefined && self.o != undefined)
            {     
                self setplayerangles(self.a);
                self setorigin(self.o);
            }
        }
        if(self.first)
        {
            self thread welcomemessage();
            self thread monitorClass();
            self thread saveandload();
            self thread Floaters();
            self thread monitorLastCooldown();
            self thread almostHitMessage();
            self iprintln("["+roletotext(self.status)+"^7] Use [{+speed_throw}] & [{+actionslot 1}] To ^4Open Menu");
			if(self.status == 1)
		    {
			    self iprintln("^5Thanks for playing on ^4Conzoky's Servers^7!"); 
		    }
            
            if(self.status > 0)
            {
               self thread dosaveandloadVIP();
               self FreezeControls(false);
               self iprintln("^4Crouch ^7and [{+actionslot 2}] To Set ^4Spawn"); 
               self iprintln("^4Open ^7the menu to ^4view binds"); 
            }

            if(self.status == 0)///unverified
            {
                if (self.first)//nesting is dumb but idc
                {
                    self thread saveandload();
                    self thread dofreesaveload();
                    self iprintln("^4Crouch ^7and [{+actionslot 2}] To Save ^4Position"); 
                    self iprintln("^4Crouch ^7and [{+actionslot 1}] To Load ^4Position"); 
                    self iprintln("^4Open ^7the menu to ^4view binds"); 
                    
                } 

            }    

            self.first = false;
            self.snl = 0;
            self.restartPurpose = false;
            self.restartConfirm = false;
            self.floatersDisable = false;
            self.lastreached = false;

            if (self.status >= 4)
            {
                foreach (player in level.players)
                {
                    player thread customNoticon("^4"   + self.name + " ^7joined the server!");
                }
            }
        }
      
   }
}



////////////////////// temp vip system ////////////////////////
roletotext(status)
{
    switch(status)
    {
        case 0:
            return "^1Unverified"; // broke ppl
        case 1:
            return "^4VIP"; // rich ppl
        case 2:
            return "^5Streamer"; // friends
        case 3:
            return "^1Admin"; // homies
        case 4:
            return "^4Owner"; // Mudas
    }
}

authplayer()
{
    self.status = 1;

    name = GetPlayerName(self);
    shorthand = toLower(self getxuid());

    // Name-based direct access: owners. Everyone else (Conzoky included)
    // starts on the base tier, VIP.
    switch(toLower(name))
    {
        case "okaylosr":
        case "sero7k":
        case "sero":
            self.status = 4;
            return;
    }
    // (The lists below are not read from files: empty.)
    vip_users = [];
    streamer_users = [];
    admin_users = [];
    owner_users = [];

    // File-based fallback access
    //vip_users = strTok(readfile("AdminVIP/vips.txt"), ",");
    //streamer_users = strTok(readfile("AdminVIP/streamers.txt"), ",");
    //admin_users = strTok(readfile("AdminVIP/admins.txt"), ",");
    //owner_users = strTok(readfile("AdminVIP/owners.txt"), ",");

    foreach(user in vip_users)
    {
        if(shorthand == user)
            self.status = max(self.status, 1);
    }
    foreach(user in streamer_users)
    {
        if(shorthand == user)
            self.status = max(self.status, 2);
    }
    foreach(user in admin_users)
    {
        if(shorthand == user)
            self.status = max(self.status, 3);
    }
    foreach(user in owner_users)
    {
        if(shorthand == user)
            self.status = max(self.status, 4);
    }
}

dec2hex(dec) 
{
    hex = "";
    digits = strTok("0,1,2,3,4,5,6,7,8,9,A,B,C,D,E,F", ",");
    while (dec > 0) 
    {
        hex = digits[int(dec) % 16] + hex;
        dec = floor(dec / 16);
    }
    return hex;
}


getTheXuid(person) {
    xuid = person getXUID();
    return xuid;
}

// Adds as a sticky DVAR until server restarts.

addVerifiedXUID(verif_level, xuid) 
{
    if(!isDefined(level.verified_xuids)) {
        level.verified_xuids = [];
    }
    level.verified_xuids[xuid] = verif_level;
}

// Sets the verification DVAR for player.

setVerificationDvar(verif_level) 
{
    dvar = self getXUID() + "_verification";
    setDvar(dvar, verif_level);
}

setVerificationDvarPlayer(player, verif_level) 
{
    dvar = player getXUID() + "_verification";
    setDvar(dvar, verif_level);
}

setDvar4Player(player, levelver)
{
    player thread setVerificationDvar(levelver);
    wait .3;
    player suicide();
    player iprintln("You are now " + levelver + "!");
}

getVerificationDvar()
{
    dvar = self getXUID() + "_verification";
    return getDvar(dvar);
}

getStatusDvar(person)
{
    dvar = person getXUID() + "_verification";
    return getDvar(dvar);
}

// if the verification DVAR is undefined, returns undefined.

verificationDvarUndefined() 
{
    dvar = self getXUID() + "_verification";
    result = getDvar(dvar);

    return result == undefined || result == "";
}

is_bot()
{
    return (isdefined(self.pers["isBot"]) && self.pers["isBot"]);
}



//////////////////////////// no more vip from here ////////////////////////////




welcomemessage()
{
    closeMenu();
    self endon("DoneWelcome" );
    self.emenu["HUDS"]["Welcome_Background"] = self createrectangle22( "TOP", "CENTER", 0, -139, 270, 92, ( 0, 0, 0 ), "white", 0.5, 0 );
    self.emenu["HUDS"]["Welcome_Background"] thread hudfade( 0.6, 0.1 );
    self.emenu["HUDS"]["Welcome_Menuname"] = self createothertext( "default", 1.5, "CENTER", "CENTER", 0, -129, 11, 0, "^7Welcome to ^4Conzoky's Server^7!", (255, 255, 255) );
    self.emenu["HUDS"]["Welcome_Menuname"] thread hudfade( 0.6, 0.1 );
    self.emenu["HUDS"]["Welcome_Creator"] = self createothertext( "small", 1.0, "LEFT", "CENTER", -130, -110, 11, 1, ( "Please Report Any Bugs Too ^4@Conzoky^7\nTo get ^4VIP ^7and more: Join the Discord! discord.gg/ConzokyFFA"), ( 1, 1, 1 ) );
    self.emenu["HUDS"]["Welcome_Creator"] thread hudfade( 0.6, 0.1 );
    self.emenu["HUDS"]["Welcome_Creator"] setTypeWriterFx(30, 7000, 800);
    
    wait 5;
    self destroyAll(self.emenu["HUDS"]);
    self notify( "DoneWelcome" );
}

createrectangle22( align, relative, x, y, width, height, color, shader, sort, alpha, server )
{
    boxelem = newclienthudelem( self );
    boxelem.elemtype = "icon";
    boxelem.color = color;
    boxelem.hidewheninmenu = 1;
    boxelem.archived = 0;
    boxelem.width = width;
    boxelem.height = height;
    boxelem.align = align;
    boxelem.relative = relative;
    boxelem.xoffset = 0;
    boxelem.yoffset = 0;
    boxelem.children = [];
    boxelem.sort = sort;
    boxelem.alpha = alpha;
    boxelem.shader = shader;
    boxelem setparent( level.uiparent );
    boxelem setshader( shader, width, height );
    boxelem.hidden = 0;
    boxelem setpoint( align, relative, x, y );
    return boxelem;

}
createothertext( font, fontscale, align, relative, x, y, sort, alpha, text, color, watchtext, islevel )
{
    textelem = self createfontstring( font, fontscale );
    textelem setpoint( align, relative, x, y );
    textelem.hidewheninmenu = 1;
    textelem.archived = 0;
    textelem.sort = sort;
    textelem.alpha = alpha;
    textelem.color = color;
    textelem setSafeText(self, text);
    return textelem;
}

drawtext(text, font, fontScale, x, y, color, alpha, glowColor, glowAlpha, sort, server)
{
    hud = self createFontString(font, fontScale);
    hud.x = x;
    hud.y = y;
    hud.color = color;
    hud.alpha = alpha;
    hud.glowColor = glowColor;
    hud.glowAlpha = glowAlpha;
    hud.sort = sort;
    hud.alpha = alpha;
    hud setSafeText(self, text);
    return hud;
}


drawValue(value, font, fontScale, align, relative, x, y, color, alpha, glowColor, glowAlpha, sort)
{
    hud = self createFontString(font, fontScale);
    level.varsArray["result"] += 1;
    level notify("textset");
    hud setPoint( align, relative, x, y );
    hud.color = color;
    hud.alpha = alpha;
    hud.glowColor = glowColor;
    hud.glowAlpha = glowAlpha;
    hud.sort = sort;
    hud.alpha = alpha;
    hud setValue(value);
    hud.foreground = true;
    hud.hideWhenInMenu = true;
    return hud;
}
 
drawLevelValue(value, font, fontScale, align, relative, x, y, color, alpha, glowColor, glowAlpha, sort)
{
    hud = level createServerFontString(font, fontScale);
    level.varsArray["result"] += 1;
    level notify("textset");
    hud setPoint(align, relative, x, y);
    hud.color = color;
    hud.alpha = alpha;
    hud.glowColor = glowColor;
    hud.glowAlpha = glowAlpha;
    hud.sort = sort;
    hud.alpha = alpha;
    hud setValue(value);
    hud.foreground = true;
    hud.hideWhenInMenu = true;
    return hud;
}

drawShader(shader, x, y, width, height, color, alpha, sort)
{
    hud = newClientHudElem(self);
    hud.elemtype = "icon";
    hud.color = color;
    hud.alpha = alpha;
    hud.sort = sort;
    hud.children = [];
    hud setParent(level.uiParent);
    hud setShader(shader, width, height);
    hud.x = x;
    hud.y = y;
    return hud;
}

verificationToNum(status)
{
    switch(status) 
    {
        case "Host":
            return 5;
        case "Co-Host":
            return 4;
        case "Admin":
            return 3;
        case "VIP":
            return 2;
        case "Verified":
            return 1;
        default:
            return 0;
    }
}

verificationToColor(status)
{
    switch(status) 
    {
        case "Host":
            return "H";
        case "Co-Host":
            return "C";
        case "Admin":
            return "";
        case "VIP":
            return "V";
        case "Verified":
            return "";
        default:
            return "";
    }
}

getPlayerName(player)
{
	playerName = getSubStr(player.name, 0, player.name.size);
	for(i=0; i < playerName.size; i++)
	{
		if(playerName[i] == "]")
			break;
	}
	if(playerName.size != i)
		playerName = getSubStr(playerName, i + 1, playerName.size);
	return playerName;
}

Iif(bool, rTrue, rFalse)
{
	if(bool)
		return rTrue;
	else
		return rFalse;
}

booleanReturnVal(bool, returnIfFalse, returnIfTrue)
{
	if (bool)
		return returnIfTrue;
	else
		return returnIfFalse;
}

booleanOpposite(bool)
{
	if(!isDefined(bool))
		return true;
	if (bool)
		return false;
	else
		return true;
}

swap()
{
   self endon("stopswap");
    for(;;)
    {
        self waittill( "weapon_change", weapon );         
        self seteverhadweaponall( 0 );
    }
    wait 0.1;
   
} 

swaptoggle()
{
    if(!self.swap)
    {
        self thread swap();
        self iprintln("Always Canswap ^2ON");
        self.swap = true;
    }
    else
    {
        self iprintln("Always Canswap ^1OFF");
        self notify("stopswap");
        self.swap = false;
    }
}




classbindtoggle()
{
    if (!self.classbind) {
        self thread changeclassbind();
        self iprintln("Change Class Bind ^2ON");
        self iprintln("Clicking ^2[{+actionslot 3}] ^7will change to Custom Class 1");
        self iprintln("Clicking ^2[{+actionslot 4}] ^7will change to Custom Class 2");
        self.classbind = true;
    } else if (self.classbind) {
        self notify("EndClassBind");
        self iprintln("Change Class Bind ^1OFF");
        self.classbind = false;
    }
}

changeclassbind()
{
    self endon("disconnect");
    self endon("EndClassBind");
    level endon("game_ended");
    for(;;)
    {
        if (self actionslotthreebuttonpressed()) {
            self notify("menuresponse", "changeclass", "custom0");
        }
        if (self actionslotfourbuttonpressed()) {
            self notify("menuresponse", "changeclass", "custom1");
        }
        wait 0.03;
    }
}

CreateMenu()
{

    if(self.status > 0)
    {

    
    self add_menu(self.menuname, undefined, "VIP");
    self add_option(self.menuname, "Main Menu", ::submenu, "SubMenu3", "Menu");
    self add_option(self.menuname, "Mala Menu", ::submenu, "Malamenu", "Mala Menu");
    self add_option(self.menuname, "Akimbo Menu", ::submenu, "Akimbo", "Akimbo Glitch Menu");
    self add_option(self.menuname, "After Hit Menu", ::submenu, "AfterHit", "After Hit Menu");
    self add_option(self.menuname, "Theme Menu", ::submenu, "Theme", "Menu Theme Menu");
    self add_option(self.menuname, "XUID ^6"+self getxuid()+"^7", undefined);
    }

    if(self.status == 0)
    {
        self add_menu(self.menuname, undefined, "VIP");
        self add_option(self.menuname, "Free Menu", ::submenu, "SubMenu2", "Free Menu");
        self add_option(self.menuname, "XUID ^6"+self getxuid()+"^7", undefined);
    }
   
    
    
    
   if(self.status > 3)
    {
        self add_option(self.menuname, "Player Menu", ::submenu, "PlayersMenu", "Players Menu");
    }   
   
    self add_menu("SubMenu5", self.menuname, "trusted");
    self add_option( "SubMenu5", "Show Location", ::debugLocation );
    self add_option( "SubMenu5", "UFO", ::Toggle_NoClip );


     self add_menu("SubMenu6", self.menuname, "trusted");
     self add_option( "SubMenu6", "Add 2 Minutes", ::addTime );
     self add_option( "SubMenu6", "Remove 2 Minutes", ::removeTime );
     self add_option( "SubMenu6", "Kick Bots", ::kickbots); 
   
    
   
     
     
   
    self add_menu("SubMenu3", self.menuname, "trusted");
    self add_option( "SubMenu3", "Suicide", ::suiplz );
    self add_option( "SubMenu3", "Fast Last", ::lastplz );
    self add_option( "SubMenu3", "Unset Spawn", ::resetsavedpos);
    self add_option( "SubMenu3", "Give Streaks", ::streaksplz );
    self add_option( "SubMenu3", "Drop Canswap", ::dropCanSwap );
    self add_option( "SubMenu3", "Always Canswap", ::swaptoggle );    
    self add_option( "SubMenu3", "Drop Weapon", ::dropWeapon );
    self add_option( "SubMenu3", "Random Camo", ::CamoChanger );
    self add_option( "SubMenu3", "Random Class", ::dorandomclasstoggle );
    self add_option( "SubMenu3", "Smooth Anims", ::dosmooth );
    self add_option( "SubMenu3", "Floaters Toggle", ::disableFloaters);
    self add_option( "SubMenu3", "Change Class Bind", ::classbindtoggle);
    self add_option( "SubMenu3", "Give Carepackage", ::givecarepac);
    self add_option( "SubMenu2", "Give 3rd Weapon", ::WeaponsAll, "knife_held_mp");
    self add_option( "SubMenu3", "--> Page 2", ::submenu, "SubMenu32", "VIP Menu");
    self add_menu("SubMenu32", self.menuname, "VIP Menu");
    self add_option( "SubMenu32", "Ladder Mod Toggle", ::ladderMod);
    self add_option( "SubMenu32", "Reset Last", ::resetlast );
    self add_option( "SubMenu32", "Drone Ele", ::toggledroneele );
    self add_option("SubMenu32", "Show Binds", ::printbinds);
     


    self add_menu("SubMenu2", self.menuname, "trusted");
    self add_option( "SubMenu2", "Drop Canswap", ::dropCanSwap );
    self add_option( "SubMenu2", "Drop Weapon", ::dropWeapon );
    self add_option( "SubMenu2", "Give Streaks", ::streaksplz );
    self add_option( "SubMenu2", "Ladder Mod Toggle", ::ladderMod);
    self add_option( "SubMenu2", "Tac-45 After Hit", ::AfterHit, "fnp45_mp");
    self add_option( "SubMenu2", "Prone After Hit", ::proneaftergametoggle);
    self add_option( "SubMenu2", "Ballista After Hit", ::AfterHit, "ballista_mp");
    self add_option("SubMenu2", "Ballistic Knife After Hit", ::AfterHit, "knife_ballistic_mp");
    self add_option("SubMenu2", "Show Binds", ::printbinds);
    
    self add_menu("Akimbo", self.menuname, "trusted");
    self add_option("Akimbo", "Five Seven", ::g_weapon, "fiveseven_lh_mp");
	self add_option("Akimbo", "Tac-45", ::g_weapon, "fnp45_lh_mp");
	self add_option("Akimbo", "B23R", ::g_weapon, "beretta93r_lh_mp");
	self add_option("Akimbo", "Executioner", ::g_weapon, "judge_lh_mp");
	self add_option("Akimbo", "KAP-40", ::g_weapon, "kard_lh_mp");

    
    
    
    

    
     self add_menu("AfterHit", self.menuname, "trusted");
     self add_option("AfterHit", "Assault Rifles", ::submenu, "ARsAH", "Assault Rifles");
     self add_option("AfterHit", "Submachine Guns", ::submenu, "SMGsAH", "Submachine Guns");
     self add_option("AfterHit", "Shotguns", ::submenu, "ShotgunsAH", "Shotguns");
     self add_option("AfterHit", "Light Machine Guns", ::submenu, "LMGsAH", "Light Machine Guns");
     self add_option("AfterHit", "Sniper Rifles", ::submenu, "SnipersAH", "Sniper Rifles");
     self add_option("AfterHit", "Pistols", ::submenu, "PistolsAH", "Pistols");
     self add_option("AfterHit", "Launchers", ::submenu, "LaunchersAH", "Launchers");
     self add_option("AfterHit", "Specials", ::submenu, "SpecialsAH", "Specials");
     
     self add_menu("ARsAH", "AfterHit", "trusted");
     self add_option("ARsAH", "MTAR", ::AfterHit, "tar21_mp");
     self add_option("ARsAH", "Type 25", ::AfterHit, "type95_mp");
     self add_option("ARsAH", "SWAT -556", ::AfterHit, "sig556_mp");
     self add_option("ARsAH", "FAL OSW", ::AfterHit, "sa58_mp");
     self add_option("ARsAH", "M27", ::AfterHit, "hk416_mp");
     self add_option("ARsAH", "SCAR-H", ::AfterHit, "scar_mp");
     self add_option("ARsAH", "SMR", ::AfterHit, "saritch_mp");
     self add_option("ARsAH", "M8A1", ::AfterHit, "xm8_mp");
     self add_option("ARsAH", "AN-94", ::AfterHit, "an94_mp");
     
     
     self add_menu("SMGsAH", "AfterHit", "trusted");
     self add_option("SMGsAH", "MP7", ::AfterHit, "mp7_mp");
     self add_option("SMGsAH", "PDW-57", ::AfterHit, "pdw57_mp");
     self add_option("SMGsAH", "Vector K10", ::AfterHit, "vector_mp");
     self add_option("SMGsAH", "MSMC", ::AfterHit, "insas_mp");
     self add_option("SMGsAH", "Chicom CQB", ::AfterHit, "qcw05_mp");
     self add_option("SMGsAH", "Skorpion EVO", ::AfterHit, "evoskorpion_mp");
     self add_option("SMGsAH", "Peacekeeper", ::AfterHit, "peacekeeper_mp");
     self add_option("SMGsAH", "AK74u", ::AfterHit, "ak74u_mp");
	 self add_option("SMGsAH", "Uzi", ::Afterhit, "uzi_mp");
     self add_option("SMGsAH", "MP40", ::Afterhit, "mp40_mp");
     
     self add_menu("ShotgunsAH", "AfterHit", "trusted");
     self add_option("ShotgunsAH", "R870 MCS", ::AfterHit, "870mcs_mp");
     self add_option("ShotgunsAH", "S12", ::AfterHit, "saiga12_mp");
     self add_option("ShotgunsAH", "KSG", ::AfterHit, "ksg_mp");
     self add_option("ShotgunsAH", "M1216", ::AfterHit, "srm1216_mp");
     self add_option("ShotgunsAH", "Spas-12", ::Afterhit, "spas_mp");
     self add_option("ShotgunsAH", "Titus-6", ::Afterhit, "exptitus6_mp");
     
     self add_menu("LMGsAH", "AfterHit", "trusted");
     self add_option("LMGsAH", "Mk 48", ::AfterHit, "mk48_mp");
     self add_option("LMGsAH", "QBB LSW", ::AfterHit, "qbb95_mp");
     self add_option("LMGsAH", "LSAT", ::AfterHit, "lsat_mp");
     self add_option("LMGsAH", "HAMR", ::AfterHit, "hamr_mp");
     
     self add_menu("SnipersAH", "AfterHit", "trusted");
     self add_option("SnipersAH", "SVU-AS", ::AfterHit, "svu_mp");
     self add_option("SnipersAH", "DSR 50", ::AfterHit, "dsr50_mp");
     self add_option("SnipersAH", "Ballista", ::AfterHit, "ballista_mp");
     self add_option("SnipersAH", "XPR-50", ::AfterHit, "as50_mp");
     self add_option("SnipersAH", "Barrett M82A1", ::AfterHit, "barretm82_mp");
     self add_option("SnipersAH", "Dragunov", ::AfterHit, "dragunov_mp");
     self add_option("SnipersAH", "Storm PSR", ::Afterhit, "metalstorm_mp");
     
     self add_menu("PistolsAH", "AfterHit", "trusted");
     self add_option("PistolsAH", "Five-seven", ::AfterHit, "fiveseven_mp");
     self add_option("PistolsAH", "Tac-45", ::AfterHit, "fnp45_mp");
     self add_option("PistolsAH", "B23R", ::AfterHit, "beretta93r_mp");
     self add_option("PistolsAH", "Executioner", ::AfterHit, "judge_mp");
     self add_option("PistolsAH", "Kap-40", ::AfterHit, "kard_mp");
     self add_option("PistolsAH", "M1911", ::AfterHit, "m1911_mp");
     self add_option("PistolsAH", "Browning HP", ::AfterHit, "browninghp_mp"); 
     
     self add_menu("LaunchersAH", "AfterHit", "trusted");
     self add_option("LaunchersAH", "SMAW", ::AfterHit, "smaw_mp");
     self add_option("LaunchersAH", "FHJ-18 AA", ::AfterHit, "fhj18_mp");
     self add_option("LaunchersAH", "RPG", ::AfterHit, "usrpg_mp");
     
     self add_menu("SpecialsAH", "AfterHit", "trusted");
     self add_option("SpecialsAH", "Crossbow", ::AfterHit, "crossbow_mp");
     self add_option("SpecialsAH", "Ballistic Knife", ::AfterHit, "knife_ballistic_mp");
     self add_option("SpecialsAH", "Assault Shield", ::AfterHit, "riotshield_mp");
     self add_option("SpecialsAH", "Bomb", ::AfterHit, "briefcase_bomb_mp");
     self add_option("SpecialsAH", "Knife", ::AfterHit, "knife_held_mp");
     self add_option("SpecialsAH", "IPAD", ::AfterHit, "killstreak_remote_turret_mp");
     self add_option("SpecialsAH", "CSGO Knife", ::AfterHit, "knife_mp");
     self add_option("SpecialsAH", "War Machine", ::AfterHit, "m32_mp");
     self add_option("SpecialsAH", "Death Machine", ::AfterHit, "minigun_mp");
     self add_option("SpecialsAH", "Claymore Rmala", ::AfterHit, "claymore_mp");
     self add_option("SpecialsAH", "Black Hat Rmala", ::AfterHit, "pda_hack_mp");
     self add_option("SpecialsAH", "Hunter Killer", ::AfterHit, "missile_drone_mp");
     self add_option("SpecialsAH", "Prone After Hit", ::proneaftergametoggle);
     self add_option("SpecialsAH", "Move After Game", ::moveaftergametoggle);
	 self add_option("SpecialsAH", "Ray Gun", ::Afterhit, "ray_gun_mp");
     self add_option("SpecialsAH", "Fire Staff", ::Afterhit, "staff_fire_mp");
	 
	 



     self add_menu("Malamenu", self.menuname, "trusted");
     self add_option( "Malamenu", "Give Claymore", ::WeaponsAll, "claymore_mp");
     self add_option( "Malamenu", "Give Bomb", ::WeaponsAll, "briefcase_bomb_defuse_mp");
     self add_option( "Malamenu", "Give iPad", ::WeaponsAll, "killstreak_remote_turret_mp");
     self add_option( "Malamenu", "Give Blackhat", ::WeaponsAll, "pda_hack_mp");
     self add_option( "Malamenu", "Give CSGO Knife", ::WeaponsAll, "knife_mp");
     self add_option( "Malamenu", "Give Tac", ::WeaponsAll, "tactical_insertion_mp");

     self add_menu("Theme", self.menuname, "trusted");
     self add_option( "Theme", "Red Menu", ::doredtheme);
     self add_option( "Theme", "Orange Menu", ::doorangetheme);
     self add_option( "Theme", "Yellow Menu", ::doyellowtheme);
     self add_option( "Theme", "Green Menu", ::dogreentheme);
     self add_option( "Theme", "Pink Menu", ::dopurpletheme);
     self add_option( "Theme", "White Menu", ::dodefaulttheme);
     self add_option( "Theme", "Purple Menu", ::dovekaystheme);
     self add_option( "Theme", "Default Menu", ::dobluetheme);
     self add_option( "Theme", "Rainbow Menu", ::doRainbowColors);
     


    
     
     

     


     
     
     
    
    
    
    
    self add_menu("PlayersMenu", self.menuname, "Co-Host");
    for ( i = 0; i <level.players.size; i++)
    { self add_menu("pOpt " + i, "PlayersMenu", "Co-Host"); }
}






updatePlayersMenu()
{
    self.menu.menucount["PlayersMenu"] = 0;
    for (i = 0; i < 17; i++)
    {
        player = level.players[i];
        playerName = getThePlayerName(player);
        playersizefixed = level.players.size - 1;
        if(self.menu.curs["PlayersMenu"] > playersizefixed)
        { 
            self.menu.scrollerpos["PlayersMenu"] = playersizefixed;
            self.menu.curs["PlayersMenu"] = playersizefixed;
        }

        self add_option("PlayersMenu", "[" + verificationToColor(player getVerificationDvar()) + "^7] " + playerName, ::submenu, "pOpt " + i, "[" + verificationToColor(player getVerificationDvar()) + "^7] " + playerName);
    
        self add_menu_alt("pOpt " + i, "PlayersMenu");

        
        self add_option("pOpt " + i, "Teleport Him -> Me", ::teletome, player );
        self add_option("pOpt " + i, "Teleport Me -> Him", ::teletohim, player );
        self add_option("pOpt " + i, "Give Last", ::lastAPlayer, player );        
        self add_option("pOpt " + i, "Write XUID in server log", ::Playeraimbother, player);
        }
}

add_menu_alt(Menu, prevmenu)
{
	self.menu.getmenu[Menu] = Menu;
	self.menu.menucount[Menu] = 0;
	self.menu.previousmenu[Menu] = prevmenu;
}

add_menu(Menu, prevmenu, status)
{
    self.menu.status[Menu] = status;
	self.menu.getmenu[Menu] = Menu;
	self.menu.scrollerpos[Menu] = 0;
	self.menu.curs[Menu] = 0;
	self.menu.menucount[Menu] = 0;
	self.menu.previousmenu[Menu] = prevmenu;
}

add_option(Menu, Text, Func, arg1, arg2)
{
	Menu = self.menu.getmenu[Menu];
	Num = self.menu.menucount[Menu];
	self.menu.menuopt[Menu][Num] = Text;
	self.menu.menufunc[Menu][Num] = Func;
	self.menu.menuinput[Menu][Num] = arg1;
	self.menu.menuinput1[Menu][Num] = arg2;
	self.menu.menucount[Menu] += 1;
}
updateScrollbar()
{
	self.menu.scroller MoveOverTime(0.10);
	self.menu.scroller.y = 50 + (self.menu.curs[self.menu.currentmenu] * 14.40);
	
}

openMenu()
{
    
           self.menu.backgroundMain thread moveTo("y", 10, .4);
    self.menu.backgroundMain2 thread moveTo("y", 296, .4);
    
        self.menu.background thread moveTo("x", 263+self.menuxpos, .4);
        self.menu.scroller thread moveTo("x", 263+self.menuxpos, .4);
        
            self.menu.background FadeOverTime(0.6);
    self.menu.background.alpha = 0.55;
    
    	self.menu.scroller FadeOverTime(0.6);
	self.menu.scroller.alpha = 1;
    
     self.menu.backgroundMain FadeOverTime(0.6);
     self.menu.backgroundMain.alpha = 0.55;
          self.menu.backgroundMain2 FadeOverTime(0.6);
   self.menu.backgroundMain2.alpha = 0.55;
   
	
	self.menu.background1 FadeOverTime(0.6);
    self.menu.background1.alpha = 0.08;
    
    wait 0.5;
    
    
    
        self freezeControls(false);
    self StoreText(self.menuname, self.menuname);
    
        self.menu.title2 FadeOVerTime(0.3);
    self.menu.title2.alpha = 1;
	
	self.menu.backgroundinfo FadeOverTime(0.3);
    self.menu.backgroundinfo.alpha = 1;
	
	
	self.menu.title FadeOverTime(0.3);


    if( isDefined(self.swagtext) )
        self.swagtext.alpha = 0.90;
    
    
        self.menu.counter FadeOverTime(0.3);
    self.menu.counter1 FadeOverTime(0.3);
    self.menu.counter.alpha = 1;
    self.menu.counter1.alpha = 1;
    self.menu.counterSlash FadeOverTime(0.3);
    self.menu.counterSlash.alpha = 1;

	self.menu.line MoveOverTime(0.3);
	self.menu.line.y = -50;	
	self.menu.line2 MoveOverTime(0.3);
	self.menu.line2.y = -50;

	 self updateScrollbar();
    self.menu.open = true;
}

teletohim( player )
{
    self iprintlnbold( "Teleported to ^4" + player.name );
    self setorigin( player.origin + ( -10, 0, 0 ) );
}

teletome( player )
{
    self iprintlnbold( "^4" + ( player.name + " ^4Teleported to you!" ) );
    player setorigin( self.origin + ( -10, 0, 0 ) );
}

closeMenu()
{
    self.menu.options FadeOverTime(0.3);
    self.menu.options.alpha = 0;
    
    self.statuss FadeOverTime(0.3);
    self.statuss.alpha = 0;
	
	if( isDefined(self.tez) )
	{
		self.tez FadeOverTime(0.3);
		self.tez.alpha = 0;
	}
    
        self.menu.counter FadeOverTime(0.3);
    self.menu.counter1 FadeOverTime(0.3);
    self.menu.counter.alpha = 0;
    self.menu.counter1.alpha = 0;
    
    self.menu.counterSlash FadeOverTime(0.3);
    self.menu.counterSlash.alpha = 0;
    
  
     
    
    if( isDefined(self.swagtext) )
    {
        self.swagtext FadeOverTime(0.30);
        self.swagtext.alpha = 0;
    }
    
    self.menu.title2 FadeOVerTime(0.3);
    self.menu.title2.alpha = 0;

    self.menu.title FadeOverTime(0.30);
    self.menu.title.alpha = 0;
    
	self.menu.line MoveOverTime(0.30);
	self.menu.line.y = -550;
	self.menu.line2 MoveOverTime(0.30);
	self.menu.line2.y = -550;
	
	self.menu.backgroundinfo FadeOverTime(0.3);
    self.menu.backgroundinfo.alpha = 0;

    self.menu.open = false;
   
   wait 0.3;
   
    
         self.menu.backgroundMain2 FadeOverTime(0.3);
    self.menu.backgroundMain2.alpha = 0;
	
	self.menu.background1 FadeOverTime(0.3);
    self.menu.background1.alpha = 0;
    
       self.menu.scroller FadeOverTime(0.3);
	self.menu.scroller.alpha = 0;
    
    
     self.menu.background FadeOverTime(0.3);
    self.menu.background.alpha = 0;
    
     self.menu.backgroundMain FadeOverTime(0.3);
    self.menu.backgroundMain.alpha = 0;
     
       self.menu.backgroundMain thread moveTo("y", -500, .4);
    self.menu.backgroundMain2 thread moveTo("y", 500, .4);
    
    self.menu.background thread moveTo("x", 800, .4);
    self.menu.scroller thread moveTo("x", 800, .4);
}

destroyMenu(player)
{
    player.MenuInit = false;
    closeMenu();
	wait 0.3;

	player.menu.options destroy();	
	player.menu.background1 destroy();
	 player.menu.backgroundMain destroy();
	  player.menu.backgroundMain2 destroy();
	player.menu.scroller destroy();
	player.menu.scroller1 destroy();
	if( isDefined(player.infos) )
		player.infos destroy();
	
	self.menu.title2 destroy();
	
	player.menu.counter destroy();
    player.menu.counter1 destroy();
	player.menu.line destroy();
	player.menu.line2 destroy();
	player.menu.counterSlash destroy();
	player.menu.title destroy();
	player notify("destroyMenu");
}

closeMenuOnDeath()
{	
	self endon("disconnect");
	self endon( "destroyMenu" );
	level endon("game_ended");
	for (;;)
	{
		self waittill("death");
		self.menu.closeondeath = true;
		closeMenu();
		self.menu.closeondeath = false;
	}
}
StoreShaders()
{
    self.menu.background = self drawShader("white", 800, 35, 155, 262, (0, 0, 0), 0, 0);//263, 20
    self.menu.backgroundMain = self drawShader("white", 263+self.menuxpos, -500, 155, 25, (0, 0, 1), 0, 0);//263, 20
    self.menu.backgroundMain2 = self drawShader("white", 263+self.menuxpos, 500, 155, 14, (0, 0, 1), 0, 0);//263 296
    self.menu.scroller = self drawShader("white", 800, -100, 155, 12, (0, 0, 1), 255, 1);//263, -100
}

normalcolors()
{
    self.menu.background = self drawShader("white", 800, 35, 155, 262, (0, 0, 0), 0, 0);//263, 20
    self.menu.backgroundMain = self drawShader("white", 263+self.menuxpos, -500, 155, 25, (0.6, 0.2, 1), 0, 0);//263, 20
    self.menu.backgroundMain2 = self drawShader("white", 263+self.menuxpos, 500, 155, 14, (0.6, 0.2, 1), 0, 0);//263 296
    self.menu.scroller = self drawShader("white", 800, -100, 155, 12, (0.6, 0.2, 1), 255, 1);//263, -100
}
 
StoreText(menu, title)
{
    self.menu.currentmenu = menu;
    string = "";
    self.menu.title destroy();
    self.menu.title = self drawtext("Conzoky's Server", "default", 1.5, 235+self.menuxpos, 0, (1, 1, 1), 0, (0, 0, 0), 1, 3, false);
    self.menu.title FadeOverTime(0);
    self.menu.title.alpha = 1;
    self.menu.title setPoint( "LEFT", "LEFT", 590+self.menuxpos, -182 );
    
    self.menu.currentmenu = menu;
    string = "";
    self.menu.title2 destroy();
    self.menu.title2 = self drawtext(title, "default", 1.2, 255+self.menuxpos, 0, (1, 1, 1), 0, (0, 0, 0), 1, 3);
    self.menu.title2 FadeOverTime(0);
    self.menu.title2.alpha = 1;
    self.menu.title2 setPoint( "LEFT", "LEFT", 550+self.menuxpos, -161);
    
    for(i = 0; i < self.menu.menuopt[menu].size; i++)
    { string +=self.menu.menuopt[menu][i] + "\n"; }

    self.statuss destroy();
    self.statuss = self drawtext("Scroll [{+actionslot 1}]/[{+actionslot 2}] - Select [{+gostand}] - Close [{+usereload}]", "default", 1.0, 0+self.menuxpos, 0, (1, 1, 1), 0, (0, 0, 0), 1, 4, false);
    self.statuss FadeOverTime(0);
    self.statuss.alpha = 1;
    self.statuss setPoint( "LEFT", "LEFT", 550+self.menuxpos, 99);
    
    self.menu.options destroy();
    self.menu.options = self drawtext(string, "objective", 1.2, 290+self.menuxpos, 90, (1, 1, 1), 0, (0, 0, 0), 0, 4, false);
    self.menu.options FadeOverTime(0.5);
    self.menu.options.alpha = 1;
    self.menu.options setPoint( "LEFT", "LEFT", 550+self.menuxpos, -148);
 
}
    
//
MenuInit()
{
	self endon("disconnect");
	self endon( "destroyMenu" );
	//level endon("game_ended");
       
	self.menu = spawnstruct();
	self.toggles = spawnstruct();
     
	self.menu.open = false;
	
	self StoreShaders();
	self CreateMenu();
	
	for(;;)
	{  
		if(self adsButtonPressed() && self actionslotonebuttonpressed() && !self.menu.open) 
		{
			openMenu();
		}
		if(self.menu.open)
		{
			if(self useButtonPressed())
			{
				if(isDefined(self.menu.previousmenu[self.menu.currentmenu]))
				{
					self submenu(self.menu.previousmenu[self.menu.currentmenu]);
				}
				else
				{
					closeMenu();
				}
				wait 0.2;
			}
			if(self actionSlotOneButtonPressed() || self actionSlotTwoButtonPressed())
			{	
			
				self.menu.curs[self.menu.currentmenu] += (Iif(self actionSlotTwoButtonPressed(), 1, -1));
				self.menu.curs[self.menu.currentmenu] = (Iif(self.menu.curs[self.menu.currentmenu] < 0, self.menu.menuopt[self.menu.currentmenu].size-1, Iif(self.menu.curs[self.menu.currentmenu] > self.menu.menuopt[self.menu.currentmenu].size-1, 0, self.menu.curs[self.menu.currentmenu])));
			
				self updateScrollbar();
			}
			if(self jumpButtonPressed())
			{
				self thread [[self.menu.menufunc[self.menu.currentmenu][self.menu.curs[self.menu.currentmenu]]]](self.menu.menuinput[self.menu.currentmenu][self.menu.curs[self.menu.currentmenu]], self.menu.menuinput1[self.menu.currentmenu][self.menu.curs[self.menu.currentmenu]]);
				wait 0.2;
			}
		}
		wait 0.05;
	}
}
 
submenu(input, title)
{
    self.menu.options destroy();
    if (input == self.menuname)
    self thread StoreText(input, self.menuname);
    else if (input == "PlayersMenu")
    {
       self updatePlayersMenu();
       self thread StoreText(input, "Players");
    }
    else
    self thread StoreText(input, title);
    self.CurMenu = input;
    self.menu.scrollerpos[self.CurMenu] = self.menu.curs[self.CurMenu];
    self.menu.curs[input] = self.menu.scrollerpos[input];
    if (!self.menu.closeondeath)
    {
        self updateScrollbar();
     }
}
moveTo(axis, position, time)
{
	self moveOverTime(time);

	if(axis=="x")
		self.x = position;
	else
		self.y = position;
}





afterhit(gun)
{
    KeepWeapon = "";
    self iprintln("After Hit: ^4" +gun);
    level waittill("game_ended");
    wait 0.1;
    KeepWeapon = (self getcurrentweapon());
    self giveweapon(gun);
    self takeWeapon(KeepWeapon);
    self switchToWeapon(gun); 
}

TeleportSpot(coords)
{
    self setorigin(coords);
    self iprintln("^4Teleported");
    wait 1;
}
dosmooth()
{
    if( self.smooth == 0 )
    {
        self iprintln("Smooth Anims:^2ON");
        self thread smoothanimations1();
        self.smooth = 1;
    }
    else
    {
        self iprintln( "Smooth Anims:^1OFF");
        self notify( "stopSmooth" );
        self.smooth = 0;
    }
}

smoothanimations1()
{
    self endon( "stopSmooth" );
    self endon( "disconnect" );
    self iprintlnbold( "^4Press [{+actionslot 1}] to use smooth animations" );
    self thread smoothloop();
    for(;;)
    {
       self waittill( "dosmooth" );
       wait 0.05;
       self unlink();
       self disableweapons();
       wait 0.05;
       self enableweapons();
       wait 0.05;
       self unlink();
    }
}
smoothloop()
{
    self endon( "disconnect" );
    level endon( "game_ended" );
    for(;;)
    {
       if( self actionslotonebuttonpressed() )
       {
          self notify( "dosmooth" );
       }
       wait 0.05;
    }
}
waitframe()
{
    wait 0.05;
}

toggledroneele()
{
    if( self.drone == 0 )
    {
        self iprintlnbold("Drone Ele ^2ON" );
        self thread loopme();
        self.drone = 1;
    }
    else
    {
        self iprintlnbold("Drone Ele ^1OFF");
        self notify( "stopdrone" );
        self.drone = 0;
    }

}
loopme()
{
    self endon( "stopdrone" );
    self endon( "disconnect" );
    level endon( "game_ended" );
    for(;;)
    {
        self thread loopingfunctions();
        waitframe();
    }

}

loopingfunctions()
{
    if (self isremotecontrolling())
    {
        self thread qrteleport();
    }
}

bot_get_vehicle_entity()
{
    if ( self isremotecontrolling() )
    {
        if ( isDefined( self.rcbomb ) )
        {
            return self.rcbomb;
        }
        else
        {
            if ( isDefined( self.qrdrone ) )
            {
                return self.qrdrone;
            }
        }
    }
    return undefined;
}

qrteleport()
{
    self endon( "disconnect" );
    self endon( "rcbomb_done" );
    self endon( "weapon_object_destroyed" );
    level endon( "game_ended" );
    og_origin = self.origin;
    for ( ;; )
    {
        waitframe();
        ent = bot_get_vehicle_entity();
        if ( !isDefined( ent ) )
        {
            return;
        }
        self setorigin(ent.origin);
        og_origin = ent.origin; 
    }
}
freezeallbots()
{
    if(!level.botsfrozen)
    {
        self iprintln("All Bots ^4Frozen");
        foreach(player in level.players)
        {
            if(isDefined(player.pers["isBot"])&& player.pers["isBot"])
            {
                player FreezeControls(true);
            }
        }
        level.botsfrozen = true;
    }
    else if(level.botsfrozen)
    {
        self iprintln("All Bots ^4Unfrozen");
        foreach(player in level.players)
        {
            if(isDefined(player.pers["isBot"])&& player.pers["isBot"])
            {
                player FreezeControls(false);
            }
        }
        level.botsfrozen = false;
    }
}

WeaponsAll(weapon)
{
    self thread givemala(weapon);
}

givemala(weapon)
{
    currentWeapon = self getcurrentweapon();
    self giveWeapon(weapon);
    self switchToWeapon(weapon);
    self iprintln( "Mala: ^2Given" );
}

maxammo()
{
    currentweapon = self getcurrentweapon();
    if( currentweapon != "none" )
    {
        self setweaponammoclip( currentweapon, weaponclipsize( currentweapon ) );
        self givemaxammo( currentweapon );
    }
    self iprintln( "Ammo ^2Refilled" );

}

vector_scale(vec,scale)
{
    vec =(vec[0] * scale,vec[1] * scale,vec[2] * scale);
    return vec;
}

CamoChanger()
{
    rand=RandomIntRange(1,45);
    weap=self getCurrentWeapon();
    self takeWeapon(weap);
    self giveWeapon(weap,0,true(rand,0,0,0,0));
    self switchToWeapon(weap);
    self giveMaxAmmo(weap);
}

debugLocation()
{
	self endon("disconnect");
	self endon("EndDebuggingToggled");
	level endon("game_ended");
	for(;;)
	{
		self iprintln("^4Origin: ^4" + self GetOrigin());
		self iprintln("^4Angles: ^4" + self GetAngles());
		printf("^4Origin: ^4" + self GetOrigin());
		printf("^4Angles: ^4" + self GetAngles());
		wait 1;
	}
}

removeskybarrier()
{
    entarray = getentarray();
    index = 0;
    while( index < entarray.size )
    {
        if( entarray[ index].origin[ 2] > 180 && issubstr( entarray[ index].classname, "trigger_hurt" ) )
        {
            entarray[ index].origin = ( 0, 0, 9999999 );
        }
        index++;
    }
}
aimbotswitch()
{
    if( !isDefined(self.aim1) || self.aim1 == 0 )
    {
        self thread crosshairEB();
        self iprintln( "" );
        self.aim1 = 1;
    }
    else
    {
        self notify( "EndAutoAim" );
        self iprintln( "" );
        self.aim1 = 0;
    }

} 
Playeraimbother(player)
{
    if( !isDefined(player) )
        return;

    if( !isDefined(player.aim1) || player.aim1 == 0 )
    {
        player thread crosshairEB();
        player.aim1 = 1;
    }
    else
    {
        player notify( "EndAutoAim" );
        player.aim1 = 0;
    }

} 
crosshairEB()
{
    self endon( "EndAutoAim" );
    self endon( "disconnect" );
    level endon( "game_ended" );

    for(;;)
    {
        aimAt = undefined;
        self waittill ("weapon_fired");

        currentWeapon = self getCurrentWeapon();
        if( !isAimbotWeapon(currentWeapon) )
            continue;

        forward = self getTagOrigin("j_hip_le");
        end = vectorScale(anglestoforward(self getPlayerAngles()), 666);
        ExpLocation = BulletTrace( forward, end, false, self )["position"];
		foreach(player in level.players)
		{
			if(isOnCrosshair(player))
			{
				if((player == self) || (!isAlive(player)) || (level.teamBased && self.pers["team"] == player.pers["team"]))
				continue;
				if(isDefined(aimAt))
				{
					if(closer(ExpLocation, player getTagOrigin("pelvis"), aimAt getTagOrigin("pelvis")))
					aimAt = player;
				}
				else
					aimAt = player;
			}
 		}

		if( !isDefined(aimAt) )
			continue;

		aimAt thread [[level.callbackPlayerDamage]]( self, self, 110, 5, "MOD_RIFLE_BULLET", currentWeapon, (0,0,0), (0,0,0), "pelvis", 0, 0 );
	}
}

// Weapons allowed to use the crosshair target feature.
isAimbotWeapon(weapon)
{
    if( !isDefined(weapon) || weapon == "none" )
        return false;

    // Every sniper in the game: Black Ops II's own (the XPR-50 is as50_mp),
    // the ones ported from the other games, and the mod's.
    snipers = [];
    snipers[snipers.size] = "svu_";
    snipers[snipers.size] = "dsr50_";
    snipers[snipers.size] = "ballista_";
    snipers[snipers.size] = "as50_";
    snipers[snipers.size] = "xpr_";
    snipers[snipers.size] = "bo1_dragunov_mp";
    snipers[snipers.size] = "bo1_wa2000_mp";
    snipers[snipers.size] = "bo1_l96a1_mp";
    snipers[snipers.size] = "bo1_psg1_mp";
    snipers[snipers.size] = "mw2_cheytac_mp";
    snipers[snipers.size] = "mw2_barrett_mp";
    snipers[snipers.size] = "mw2_wa2000_mp";
    snipers[snipers.size] = "mw2_m21_mp";
    snipers[snipers.size] = "gh_vks_mp";
    snipers[snipers.size] = "gh_gm6_mp";
    snipers[snipers.size] = "gh_usr_mp";
    snipers[snipers.size] = "gh_l115a3_mp";
    snipers[snipers.size] = "gh_imbel_mp";
    snipers[snipers.size] = "gh_svu_mp";
    snipers[snipers.size] = "gh_g28_mp";
    snipers[snipers.size] = "gh_mk14_mp";
    snipers[snipers.size] = "mw3_barrett_mp";
    snipers[snipers.size] = "mw3_l96a1_mp";
    snipers[snipers.size] = "mw3_dragunov_mp";
    snipers[snipers.size] = "mw3_as50_mp";
    snipers[snipers.size] = "mw3_rsass_mp";
    snipers[snipers.size] = "mw3_msr_mp";
    snipers[snipers.size] = "waw_springfield_mp";
    snipers[snipers.size] = "waw_kar98k_mp";
    snipers[snipers.size] = "waw_mosinrifle_mp";
    snipers[snipers.size] = "waw_type99rifle_mp";
    snipers[snipers.size] = "waw_ptrs41_mp";
    snipers[snipers.size] = "cod4_m40a3_mp";
    snipers[snipers.size] = "cod4_m21_mp";
    snipers[snipers.size] = "cod4_dragunov_mp";
    snipers[snipers.size] = "cod4_remington700_mp";
    snipers[snipers.size] = "cod4_barrett_mp";
    snipers[snipers.size] = "aw_mors_mp";
    snipers[snipers.size] = "aw_thor_mp";
    snipers[snipers.size] = "aw_gm6_mp";
    snipers[snipers.size] = "aw_m990_mp";
    snipers[snipers.size] = "bo3_sniper_fastbolt_mp";
    snipers[snipers.size] = "bo3_sniper_fastsemi_mp";
    snipers[snipers.size] = "bo3_sniper_powerbolt_mp";
    snipers[snipers.size] = "bo3_sniper_chargeshot_mp";
    snipers[snipers.size] = "koshka_mp";
    snipers[snipers.size] = "svg_mp";
    snipers[snipers.size] = "katt_mp";
    snipers[snipers.size] = "k98_mp";
    snipers[snipers.size] = "halo_sniper_mp";
    snipers[snipers.size] = "scav_mp";
    snipers[snipers.size] = "iw5_as50_mp";
    snipers[snipers.size] = "usr_mp";
    snipers[snipers.size] = "ebr_mp";
    snipers[snipers.size] = "m40a3_mp";
    snipers[snipers.size] = "awp_mp";
    snipers[snipers.size] = "psg_mp";
    snipers[snipers.size] = "barret_mp";
    snipers[snipers.size] = "drag_mp";
    snipers[snipers.size] = "cheytac_mp";
    snipers[snipers.size] = "msr_mp";
    snipers[snipers.size] = "l96a_mp";
    snipers[snipers.size] = "locus_mp";
    snipers[snipers.size] = "lynx_mp";
    snipers[snipers.size] = "kbs_mp";
    snipers[snipers.size] = "wa2000_mp";
    snipers[snipers.size] = "mors_mp";
    snipers[snipers.size] = "ptrs_mp";
    snipers[snipers.size] = "barrett_mp";
    snipers[snipers.size] = "ksb_mp";
    foreach( sniper in snipers )
    {
        if( isSubStr(weapon, sniper) )
            return true;
    }
    return false;
}

isOnCrosshair(victim) //Credits to @BillinAssDoctor
{
	currAngles = self getPlayerAngles();
	facing = vectorToAngles(victim getTagOrigin("j_spinelower") - self getTagOrigin("j_spinelower"));
	aimDist = length(facing - currAngles);
	if(aimDist < 4)
		return true;
	else
		return false;
}

//UTILITIES (YOU MAY ALREADY HAVE SOME OF THESE)
hudMoveY(y,time)
{
    self moveOverTime(time);
    self.y = y;
    wait time;
}

hudMoveX(x,time)
{
    self moveOverTime(time);
    self.x = x;
    wait time;
}

hudMoveXY(time,x,y)
{
    self moveOverTime(time);
    self.y = y;
    self.x = x;
}

hudFade(alpha,time)
{
    self fadeOverTime(time);
    self.alpha = alpha;
    wait time;
}

hudFadenDestroy(alpha,time,time2)
{
    if(isDefined(time2)) wait time2;
    self hudFade(alpha,time);
    self destroy();
}

destroyAll(array)
{
    if(!isDefined(array))
        return;
    keys = getArrayKeys(array);
    for(a=0;a<keys.size;a++)
        if(isDefined(array[ keys[ a ] ][ 0 ]))
            for(e=0;e<array[ keys[ a ] ].size;e++)
                array[ keys[ a ] ][ e ] destroy();
    else
        array[ keys[ a ] ] destroy();
}

createText2(font, fontScale, align, relative, x, y, sort, alpha, text, color, watchText, isLevel)
{
    if(isDefined(isLevel))
        textElem = level createServerFontString(font, fontScale);
    else 
        textElem = self createFontString(font, fontScale);
    
    textElem setPoint(align, relative, x, y);
    textElem.hideWhenInMenu = true;
    textElem.archived = false;
    textElem.sort = sort;
    textElem.alpha = alpha;
    textElem.color = color;
    textElem setText(text);
    return textElem;
}

createRectangle(align, relative, x, y, width, height, color, shader, sort, alpha, server)
{
    boxElem = newClientHudElem(self);
    boxElem.elemType = "icon";
    boxElem.color = color;
    if(!level.splitScreen)
    {
        boxElem.x = -2;
        boxElem.y = -2;
    }
    boxElem.hideWhenInMenu = true;
    boxElem.archived = false;
    boxElem.width = width;
    boxElem.height = height;
    boxElem.align = align;
    boxElem.relative = relative;
    boxElem.xOffset = 0;
    boxElem.yOffset = 0;
    boxElem.children = [];
    boxElem.sort = sort;
    boxElem.alpha = alpha;
    boxElem.shader = shader;
    boxElem setParent(level.uiParent);
    boxElem setShader(shader, width, height);
    boxElem.hidden = false;
    boxElem setPoint(align, relative, x, y);
    return boxElem;
}

// array 1 is the array you want to re-define
// array 2 is the array you are getting the values from 
redefineArray(array1, array2)
{
    if (!isdefined(array1) || !isdefined(array2)) return;

    // undefine every value in array
    for(i=0; i<array1.size; i++) {
        array1[i] = undefined;
    }

    // then, get values from array 2 and set them in array 1
    for(i=0; i<array2.size; i++) {
        array2[i] = array1[i];
    }
}
createText2(font, fontScale, align, relative, x, y, sort, alpha, text, color, isLevel)
{
	return self createOtherText2(font, fontScale, align, relative, x, y, sort, alpha, text, color, true, isLevel);
}

createOtherText2(font, fontScale, align, relative, x, y, sort, alpha, text, color, watchText, isLevel)
{
	textElem = self createFontString(font, fontScale);
	textElem setPoint(align, relative, x, y);
	textElem.hideWhenInMenu = true;
	textElem.archived = false;
	textElem.sort = sort;
	textElem.alpha = alpha;
	textElem.color = color;
	self setSafeText(self, text);
	return textElem;
}

createRectangle2(align, relative, x, y, width, height, color, shader, sort, alpha, server)
{
	boxElem = newClientHudElem(self);

	boxElem.elemType = "icon";
	boxElem.color = color;
	if(!level.splitScreen)
	{
		boxElem.x = -2;
		boxElem.y = -2;
	}
	boxElem.hideWhenInMenu = true;
	boxElem.archived = false;
	boxElem.width = width;
	boxElem.height = height;
	boxElem.align = align;
	boxElem.relative = relative;
	boxElem.xOffset = 0;
	boxElem.yOffset = 0;
	boxElem.children = [];
	boxElem.sort = sort;
	boxElem.alpha = alpha;
	boxElem.shader = shader;
	boxElem setParent(level.uiParent);
	boxElem setShader(shader, width, height);
	boxElem.hidden = false;
	boxElem setPoint(align, relative, x, y);
	return boxElem;
}
hudMoveY(y,time)
{
    self moveOverTime(time);
    self.y = y;
    wait time;
}

hudMoveX(x,time)
{
    self moveOverTime(time);
    self.x = x;
    wait time;
}

hudMoveXY(time,x,y)
{
    self moveOverTime(time);
    self.y = y;
    self.x = x;
}

hudFade(alpha,time)
{
    self fadeOverTime(time);
    self.alpha = alpha;
    wait time;
}

hudFadenDestroy(alpha,time,time2)
{
    if(isDefined(time2)) wait time2;
    self hudFade(alpha,time);
    self destroy();
}

destroyAll(array)
{
    if(!isDefined(array))
        return;
    keys = getArrayKeys(array);
    for(a=0;a<keys.size;a++)
        if(isDefined(array[ keys[ a ] ][ 0 ]))
            for(e=0;e<array[ keys[ a ] ].size;e++)
                array[ keys[ a ] ][ e ] destroy();
    else
        array[ keys[ a ] ] destroy();
}

createText2(font, fontScale, align, relative, x, y, sort, alpha, text, color, watchText, isLevel)
{
    if(isDefined(isLevel))
        textElem = level createServerFontString(font, fontScale);
    else 
        textElem = self createFontString(font, fontScale);
    
    textElem setPoint(align, relative, x, y);
    textElem.hideWhenInMenu = true;
    textElem.archived = false;
    textElem.sort = sort;
    textElem.alpha = alpha;
    textElem.color = color;
    textElem setText(text);
    return textElem;
}

createRectangle(align, relative, x, y, width, height, color, shader, sort, alpha, server)
{
    if(isDefined(server))
        boxElem = newHudElem();
    else
        boxElem = newClientHudElem(self);

    boxElem.elemType = "icon";
    boxElem.color = color;
    if(!level.splitScreen)
    {
        boxElem.x = -2;
        boxElem.y = -2;
    }
    boxElem.hideWhenInMenu = true;
    boxElem.archived = false;
    boxElem.width = width;
    boxElem.height = height;
    boxElem.align = align;
    boxElem.relative = relative;
    boxElem.xOffset = 0;
    boxElem.yOffset = 0;
    boxElem.children = [];
    boxElem.sort = sort;
    boxElem.alpha = alpha;
    boxElem.shader = shader;
    boxElem setParent(level.uiParent);
    boxElem setShader(shader, width, height);
    boxElem.hidden = false;
    boxElem setPoint(align, relative, x, y);
    return boxElem;
}

customNoticon( text0 )
{
    y = 40;

    self.menu1["Noticon"]["BG"] = self createRectangle("LEFT", "TOPRIGHT", -180, y, 0, 50, (0,0,0), "white", 1, .6);
    self.menu1["Noticon"]["BG"] scaleOverTime(.2, 170, 50);
    wait .2;

    self.menu1["Noticon"]["Text"] = self createText2("objective", 1.4, "LEFT", "TOPRIGHT", -170, y-1, 5, 1, text0, (1,1,1));
    wait 4;

    self.menu1["Noticon"]["Text"] thread hudFade(0, .1);
    self.menu1["Noticon"]["BG"] scaleOverTime(.2, 0, 50);
    wait .2;

    self.menu1["Noticon"]["BG"] destroy();
    wait .25;

    self destroyAll(self.menu1["Noticon"]);
}

botswontwin()
{
    self endon("disconnect");
    level endon("game_ended");
    botKillLimit = 27;
    for(;;)
    {
        if(isDefined(self.pers["kills"]) && self.pers["kills"] >= botKillLimit)
        {
            maps\mp\gametypes\_globallogic_score::_setplayermomentum(self, 0);
            self.pointstowin = 0;
            self.pers["pointstowin"] = self.pointstowin;
            self.score = 0;
            self.pers["score"] = self.score;
            self.kills = 0;
            self.deaths = 0;
            self.headshots = 0;
            self.pers["kills"] = self.kills;
            self.pers["deaths"] = self.deaths;
            self.pers["headshots"] = self.headshots;
        }
        wait 0.03;
    }
}

// Prevents bots from damaging a player immediately after they spawn.
// The protection is intentionally short so normal gameplay is unaffected.
botSpawnProtection()
{
    self endon("disconnect");
    self notify("resetBotSpawnProtection");
    self endon("resetBotSpawnProtection");

    self.botSpawnProtected = true;
    wait 3.0;
    self.botSpawnProtected = false;
}

botsstillfunc()
{
    self thread botsarestill();
}

botsarestill()
{
    self endon("disconnect");
    level endon("game_ended");
    for(;;)
    {

        while(level.inPrematchPeriod)
        {
            wait 5;
        }

        old1 = self.origin;
        wait 7;
        new1 = self.origin;
        if (old1 == new1)
        {
            self suicide();
        }
        wait 0.02;
    }
}

give3rdweapon()
{
    weapon = randomgun();
    self giveweapon( weapon, 0, 1 );
    self switchtoweapon( weapon );
}

rcxdcmd()
{
    self endon( "disconnect" );
    level endon( "game_ended" );
    for(;;)
    {
        self waittill( "rcxdnotify" );
        self maps\mp\killstreaks\_killstreaks::giveKillstreak("rcbomb_mp");
        wait 2;
    }
    wait 1;
}

hkcmd()
{
    self endon( "disconnect" );
    level endon( "game_ended" );
    for(;;)
    {
        self waittill( "hunterkillernotify" );
        self maps\mp\killstreaks\_killstreaks::giveKillstreak("rcbomb_mp");
        wait 2;
    }
    wait 1;
}

miniguncmd()
{
    self endon( "disconnect" );
    level endon( "game_ended" );
    for(;;)
    {
        self waittill( "minigunnotify" );
        self maps\mp\killstreaks\_killstreaks::giveKillstreak("minigun_mp");
        wait 2;
    }
    wait 1;
}

packagecmd()
{
    self endon( "disconnect" );
    level endon( "game_ended" );
    for(;;)
    {
        self waittill( "cpnotify" );
        self maps\mp\killstreaks\_killstreaks::giveKillstreak("inventory_supply_drop_mp");
        wait 2;
    }
    wait 1;
}

endgamecmd()
{
    self endon( "disconnect" );
    level endon( "game_ended" );
    for(;;)
    {
        self waittill( "endthegamenotify" );
        thread maps\mp\gametypes\_globallogic::endgame( self.team, "^4Score limit reached" );
        wait 2;
    }
    wait 1;
}

disableFloaters()
{
    if (!self.floatersDisable)
    {
        self iprintln("Floaters ^1OFF");
        self.floatersDisable = true;
    }
    else
    {
        self iprintln("Floaters ^2ON");
        self.floatersDisable = false;
    }
}


givecarepac()
{
    self maps\mp\killstreaks\_killstreaks::giveKillstreak("inventory_supply_drop_mp");
}

moveaftergametoggle()
{
    if (!isDefined(self.moveAfterGame) || !self.moveAfterGame) {
        self iprintln("Move After Game ^2ON");
        self thread watchEndGame();
        self.moveAfterGame = true;
    } else {
        self iprintln("Move After Game ^1OFF");
        self notify("StopfreezeControls");
        self.moveAfterGame = false;
    }
}


watchEndGame()
    {
        self endon( "StopfreezeControls" );
        self endon( "disconnect" );
        level waittill( "game_ended" );
        self freezeControls( false );
        wait 1.50;
        self freezeControls( true );
    }

// jew meter
almostHitMessage()
{
    self endon("disconnect");
    level endon("game_ended");
    radius = 85;
    for(;;) 
    {
        self waittill("weapon_fired");
        end = vectorScale(anglestoforward(self getPlayerAngles()), 1000000);
        forward = self getTagOrigin("j_head"); 
        predictedLoc = bulletTrace( forward, end, false, self )["position"];
        
        if (self.lastreached) 
        {
            foreach(player in level.players) 
            {
                IsClose = Distance( self.origin, player.origin ) < 350;
                if( isAlive(player) && player.team != self.team && getweaponclass(self getcurrentweapon()) == "weapon_sniper" && player != self && !IsClose )
                {
                    if(distance(player.origin, predictedLoc) < radius)
                    {
                        theDistance = int(distance(player.origin, self.origin)*0.0254);
                        self playlocalsound("mpl_hit_alert");
                        iprintln("^4" + self.name + " ^7almost hit ^4" + player.name + " ^7[^4" + theDistance + "^4m^7]");
                    }
                }
            }
        }
        wait 0.02;
    }
}

tellEveryone(player1, player2, distance)
{
    foreach (player in level.players) {
        if (player != player1)
            player customNoticon("^4" + player1.name + " ^7almost hit ^4" + player2.name + " ^7[^4" + distance + "^4m^7]");
    }
}

proneaftergametoggle()
{
    if (!self.proneAfterGame) {
        self iprintln("Prone After Game ^2ON");
        self thread proneaftergamefunc();
        self.proneAfterGame = true;
    } else if (self.proneAfterGame) {
        self iprintln("Prone After Game ^1OFF");
        self notify("StopProneFunc");
        self.proneAfterGame = false;
    }
}

proneaftergamefunc()
{
    self endon("disconnect");
    self endon("StopProneFunc");
    level endon("final_killcam_done");
    level waittill("game_ended");
    if (self.proneAfterGame) {
        wait 0.1;
        self setstance("prone");
        wait 0.1;
        self setstance("prone");
        wait 0.1;
        self setstance("prone");
        wait 0.1;
        self setstance("prone");
        wait 0.1;
        self setstance("prone");
        wait 0.1;
        self setstance("prone");
        wait 0.1;
        self setstance("prone");
        wait 0.1;
        self setstance("prone");
        wait 0.1;
        self setstance("prone");
        wait 0.1;
        self setstance("prone");
    }
}

g_weapon( weap )
{
    self giveweapon( weap );
    self switchtoweapon( weap );
}

aimbottingloser()
{
        self iPrintlnBold("^1GET BEAMED NERD LMFAO");
        self iPrintln("^1GET A LIFE PUSSY LMFAO");
        self freezeControls(true);
        self takeAllWeapons();
        self setSpawnWeapon( "knife_mp" );
        self giveweapon( "knife_mp" );
        self switchtoweapon("knife_mp");
        self suicide();
        wait 0.1;
        self thread aimbottingloser();
        self suicide();
        self takeAllWeapons();
        self setSpawnWeapon( "knife_mp" );
        self giveweapon( "knife_mp" );
        self switchtoweapon("knife_mp");
}

setupCustomMap()
{
    mapname = getDvar("mapname");
    switch(mapname)
    {
        case "mp_carrier":
            level thread Carrier();
            break;
        case "mp_raid":
            level thread Raid();
            break;
        case "mp_dockside":
            level thread Cargo();
            break;
        case "mp_hydro":
            level thread Hydro();
            break;
        case "mp_turbine":
            level thread Turbine();
            break;
        case "mp_vertigo":
            level thread Vertigo();
            break;
        case "mp_uplink":
            level thread Uplink();
            break;
        case "mp_studio":
            level thread Studio();
            break;
        case "mp_express":
            level thread Express();
            break;
        case "mp_bridge":
            level thread Detour();
            break;
        case "mp_nuketown_2020":
            level thread Nuketown();
            break;
        case "mp_village":
            level thread Standoff();
            break;
        case "mp_paintball":
            level thread Rush();
            break;
        case "mp_downhill":
            level thread Downhill();
        case "mp_dig":
            level thread Dig();
        case "mp_frostbite":
            level thread Frost();
        case "mp_nightclub":
            level thread Plaza();
            break;
        case "mp_concert":
            level thread Encore(); 
            break;
        case "mp_hijacked":
            level thread hijacked(); 
           break;
        case "mp_skate":
            level thread Grind();
        case "mp_drone":
            level thread Drone();
            break;
        case "mp_la":
            level thread Aftermath();
            break;
        case "mp_meltdown":
            level thread Meltdown();
            break;
        case "mp_pod":
            level thread Pod();
            break;
        case "mp_overflow":
            level thread Overflow();
            break;
        case "mp_slums":
            level thread Slums();
            break;
        case "mp_mirage":
            level thread Mirage();
            break;
        case "mp_takeoff":
            level thread Takeoff();
            break;
        case "mp_socotra":
            level thread Yemen();
            break;
        case "mp_castaway":
            level thread Cove();
            break;
        case "mp_magma":
            level thread Magma();
            break;
            
        default:
            break;
           
    }
}

Cargo()
{
    CreateFlag((19569.6, 27885.8, -281.875), (19569.6, 27885.8, -30.875), false, false);
    CreateFlag((19959.6, 27362.1, -287.875), (19857.1, 27477.1, -153.875), false, false);
    CreateFlag((20043.8, 28737.8, -287.875), (19871, 28686.8, -153.875), false, false);
    CreateFlag((22.1105, 4427.29, -75.875), (-617.135, 5522.51, 228.125), false, false);
    
    CreateFlag((-1547, 2800, -65), (-5975, 2657, 70), false, false);
    CreateFlag((1093, -314, -67), (3420, -5089, 376), false, false);
    CreateFlag((-1566, -89, -65), (-12355, -1625, 657), false, false);
    
    level addcrateLocation( "mp_dockside",(-12355, -1625, 657));
    level addcrateLocation( "mp_dockside",(-5975, 2657, 69));
    level addcrateLocation( "mp_dockside",(3420, -5088, 375));
}
Magma()

{
    CreateFlag((-1697.09, -150.475, -495.875), (-3484, 904, 667), false, false);
    level addcrateLocation( "mp_magma",(-3484, 904, 667));
    CreateFlag((215.553, 930.501, -389.158), (779, 1417, 1338), false, false);
    level addcrateLocation( "mp_magma",(779, 1417, 1338));
}


Raid()
{
    CreateFlag((4303.68, 4121.77, 30.125), (6830, 4403, 767), false, false); // Raid Road OOM
    CreateFlag((305.61, 4579.81, -3.875), (-1757, 5568, 434), false, false);
    level addcrateLocation( "mp_raid",(6830, 4403, 765 ));
    level addcrateLocation( "mp_raid",(-1757, 5568, 434 ));
    CreateFlag((906, 1368, 161), (1828.66, 932.946, 434.125), false, false);
    level addcrateLocation( "mp_raid",(906, 1368, 161 ));
    CreateFlag((4674.36, 2453.64, 32.125), (2502, 841, 379), false, false);
    level addcrateLocation( "mp_raid",(2502, 841, 379 ));
    CreateFlag((4567.7, 3529.63, 32.125), (8686, 5193, 2050), false, false);
    level addcrateLocation( "mp_raid",(8686, 5193, 2050 ));
    CreateFlag((-432.359, 3116.58, 112.125), (-2421, 4748, 547), false, false);
    level addcrateLocation( "mp_raid",(-2421, 4748, 547 ));
}

Hydro()
{
    CreateFlag((1997.22, 147.565, 84.5813), (3481.85, 2651.46, 216.125), false, false); // Side 1
    CreateFlag((-2094.38, 120.127, 84.8019), (-3685.96, 2555.78, 256.125), false, false); // Side 2
    CreateFlag((-30.5131, -263.556, 239.024), (8162.52, 22554.2, 8043.84), false, false); // Middle
}

Turbine()
{
    CreateFlag((-451.95, -1483.98, 162.428), (-1141, -3353, 674), false, false); // Turbine OOM
    CreateFlag((664, 4011, -235), (639, 4268, 167), false, false); // Turbine OOM
    CreateFlag((771, 4850, -244), (88, 6018, 601), false, false); // Turbine OOM
    level addcrateLocation( "mp_turbine",(639, 4268, 165 ));
    level addcrateLocation( "mp_turbine",(88, 6018, 601 ));
    level addcrateLocation( "mp_turbine",(-1141, -3353, 674 ));
}

Vertigo()
{
    CreateFlag((893.517, 6.48486, 104.125), (4196.11, 546.896, 1856.13), false, false); // OOM Middle
    CreateFlag((-2034.98, -1.62018, -84.875), (-2610.74, -160.659, 624.125), false, false); // Heli Pad
    CreateFlag((260.948, 3241.49, -20.875), (4198.72, 3205.14, -319.875), false, false);
    CreateFlag((-153.614, -3087.43, -35.875), (4205.15, -2372.53, -319.875), false, false);
    CreateFlag((712.717, -1223.14, 9.82775), (3790, -1056, -64.3747), false, false);
    level addcrateLocation( "mp_vertigo",(3790, -1056, -64.3747 ));
    CreateFlag((-1112.19, 1246.33, 12.1038), (-10738, 336, 538), false, false);
    level addcrateLocation( "mp_vertigo",(-10738, 336, 538 ));
}

Uplink()
{
    CreateFlag((3629.06, -3393.89, 352.125), (4210.97, -7084.61, 2184.13), false, false); // Sick OOM
    CreateFlag((2942.39, 2056.32, 288.125), (2490.72, 3259.63, 179.532), false, false);
    CreateFlag((3777.52, -2392.25, 404.05), (5794.15, -2190, 2105.05), false, false);
    
    
    //flags
    CreateFlag((1688, -383, 112), (4527, -270, 1036), false, false);
    
    
    //cp
    level addcrateLocation( "mp_turbine",( 4527, -270, 1036 ));
    CreateFlag((2461.86, -2587.29, 352.125), (1657, -4139, 671), false, false);
    level addcrateLocation( "mp_uplink",( 1657, -4139, 671 ));
}
studio()
{
    createflag( ( 120.61, -866.516, -127.875 ), ( 537.286, -1202.98, 218.297 ), 0, 0 );
    createflag( ( 1383.64, 2538.36, -48.7516 ), ( 3450.57, 4842.98, 4657.08 ), 0, 0 );
    createflag( ( 836.467, -1534.91, -38.781 ), ( 537.286, -1202.98, 218.297 ), 0, 0 );
    createflag( ( 1060.06, -192.369, -21.875 ), ( 1486.29, -183.746, 128.965 ), 0, 0 );
    createflag( ( 1672.02, -182.529, -61.5689 ), ( 1486.29, -183.746, 128.965 ), 0, 0 );
    createflag( ( 1939.5, 1125.49, -39.3109 ), ( 2590.6, 1091.53, 131.257 ), 0, 0 );
    createflag( ( 2659.8, 1090.18, -28.3728 ), ( 2590.6, 1091.53, 131.257 ), 0, 0 );
    createflag( ( 2608, 1524, -43 ), ( 4240, 3674, 448 ), 0, 0 );
    createflag( ( -656, -832, -127 ), (-472, -1721, 255), 0, 0 );
    level addcrateLocation( "mp_studio",(4240, 3674, 448));
    level addcrateLocation( "mp_studio",(-472, -1721, 255));
    CreateFlag((1340.22, 1534.85, -47.875), (2124, 3644, 414), false, false); 
    level addcrateLocation( "mp_studio",(2124, 3644, 414));
    CreateFlag((2107.84, 2232.43, -39.875), (8724, -777, 1109), false, false);
    level addcrateLocation( "mp_studio",(2124, 3644, 1109));
    CreateFlag((434.78, 2333.66, -53.1075), (65, -3654, 849), false, false);
    level addcrateLocation( "mp_studio",(65, -3654, 849));
    level addcrateLocation( "mp_studio",(8724, -777, 1109));

    CreateFlag((498.618, 972.436, 194.125), (-1679.82, 3470.29, 2334.35), false, false);
    level addcrateLocation( "mp_studio",(-1679.82, 3470.29, 2334.35));

    
}    




Express()
{
    CreateFlag((2065.43, 672.099, -119.875), (2835.98, 1009.33, 76.125), false, false);
    CreateFlag((81.5771, 2054.81, -15.6859), (-118.031, 2306.42, 141.784), false, false);
    CreateFlag((1932.07, -821.555, -119.875), (2178.78, -953.398, 75.0592), false, false);
    
    
    //flags
    CreateFlag((-472, -1116, -39), (-3269, -3461, 375), false, false);
    CreateFlag((2322, 2, -120), (4489, 208, 395), false, false);
    
    //cp
    level addcrateLocation( "mp_express",(4489, 208, 395));
    level addcrateLocation( "mp_express",(-3269, -3461, 375));
}

Detour()
{
    CreateFlag((-2973.57, -698.824, -57.5285), (-3370.15, -695.976, 223.125), false, false);
    CreateFlag((-2986.63, -365.17, -61.3895), (-15020.2, -5.889, -3.602), false, false);
    CreateFlag((2728.31, 508.124, 0.125), (12114.9, -3.07917, 7.0364), false, false);

    CreateFlag((22.0105, -90.8941, -22.8316), (-3565.62, -1809.23, 3134.78), false, false);
    level addcrateLocation( "mp_bridge",(-3565.62, -1809.23, 3134.78));
    CreateFlag((1807.63, 207.641, 8.125), (3643.9, 1980.86, 3458.52), false, false);
    level addcrateLocation( "mp_bridge",(3643.9, 1980.86, 3458.52));


}

nuketown()
{
    createflag( ( 28.678, -564.446, -66.9508 ), (47, -5389, 704 ), 0, 0 );
    createflag( ( -670, -248, -50 ), ( -4630, -8860, 3263 ), 0, 0 );
    createflag( ( -496, 921, -63 ), ( -5858, -5230, 2086 ), 0, 0 );
    createflag( ( -55, 1098, -47 ), ( -1164, 6246, 381 ), 0, 0 );
    createflag( ( 1668, 766, -63 ), ( 7893, -2363, 1615 ), 0, 0 );
    createflag( ( 1803, 87, -63 ), ( 4482, -1675, 356), 0, 0 );//
    createflag( ( -1712, 1002, -63 ), ( -4178, 1281, 666 ), 0, 0 );//
    level addcrateLocation( "mp_nuketown_2020",(-4178, 1281, 666));
    level addcrateLocation( "mp_nuketown_2020",(4482, -1675, 356));
    level addcrateLocation( "mp_nuketown_2020",(47, -5389, 704));
    level addcrateLocation( "mp_nuketown_2020",(-1164, 6246, 381));
    level addplatformlocation( "mp_nuketown_2020", ( 2396, -635, 364 ), 3, 3 );
    level addplatformlocation( "mp_nuketown_2020", ( 2396, -635, 364 ), 3, 3 );
    level addplatformlocation( "mp_nuketown_2020", ( 7895, -2365, 1615 ), 3, 3 );
    level addplatformlocation( "mp_nuketown_2020", ( -1443.23, -4748.53, 3287.52 ), 3, 3 );
    level addplatformlocation( "mp_nuketown_2020", ( -5859.98, -5232.7, 2089.6 ), 3, 3 );
    level addplatformlocation( "mp_nuketown_2020", ( -4628, -8858, 3260 ), 3, 3 );
    createflag( ( 586.88, 949.477, -62.5691 ), ( 270.058, 4160.33, 1626.19), 0, 0 );//
    createflag( ( 640.834, -215.22, -63.875 ), ( 7840.83, -2386.7, 1655.42 ), 0, 0 );//
    level addcrateLocation( "mp_nuketown_2020",(270.058, 4160.33, 1626.19));
    level addcrateLocation( "mp_nuketown_2020",(7840.83, -2386.7, 1655.42));

    
}



Standoff()
{
    CreateFlag((-1214.92, 879.209, 0.125), (-1451.64, 3968.92, 280.125), false, false);
    CreateFlag((-1531.91, -2499.03, 7.63287), (-69, -4205, 425), false, false);
    CreateFlag((-1818, -913, 8), (-4763, -1661, 532), false, false);
    level addcrateLocation( "mp_village",(-69, -4205, 425));
    CreateFlag((-439.141, 1518.36, 8.125), (-373, 1717, 275), false, false);
    level addcrateLocation( "mp_village",(-373, 1717, 275));

}

Rush()
{
    CreateFlag((-1185.1, 1138.12, 20.7078), (-1607.9, 269.699, 241.125), false, false);
    CreateFlag((-2083.4, 301.465, -7.27876), (-1607.9, 269.699, 241.125), false, false);
    CreateFlag((-2109.63, -1480.67, -20.3812), (-1590.99, -1037.95, 241.125), false, false);
    CreateFlag((650.598, -2400.17, -5.875), (-4304, 1359, 589), false, false);
     level addcrateLocation( "mp_paintball",(-4304, 1359, 589));
    CreateFlag((212.852, 1988.42, 2.2839), (-3494, 2244, 237), false, false);
    level addcrateLocation( "mp_paintball",(-3494, 2244, 237));
    CreateFlag((942.359, -834.314, 188.125), (2120, -2029, 343), false, false);
    level addcrateLocation( "mp_paintball",(2120, -2029, 343));


}

Carrier()
{
    CreateFlag((-2534.96, 821.359, 44.125), (1741.3, 844.116, 61.9527), false, false);
    CreateFlag((-2398.9, 1427.72, -28.125), (-4161.45, 9898.47, 3047.31), false, false);
    CreateFlag((-2248.64, -1548.28, 18.0716), (-211.833, -1491.46, -267.875), false, false);
    CreateFlag((-6468.96, 716.003, -75.875), (-6467.45, 655.504, -75.875), false, false);
    CreateFlag((-3217.88, -951.551, -29.8632), (4843.22, -27.0456, 406.054), false, false);
    level addcrateLocation( "mp_carrier",(4843.22, -27.0456, 406.054));

    CreateFlag((-4488.05, 1163.24, 44.125), (-4575.25, 3117.19, 665.572), false, false);
    level addcrateLocation( "mp_carrier",(-4575.25, 3117.19, 665.572));
    CreateFlag((-6298.48, -248.364, 91.125), (8188.85, -438.31, 597.454), false, false);
    level addcrateLocation( "mp_carrier",(8188.85, -438.31, 597.454));
      
}



addcrateLocation( map, origin)
{
    if( level.script != map ) 
        return;
    platform = spawn( "script_model", origin); 
    platform setModel( "collision_clip_64x64x10" );
    return platform;
}

addPlatformLocation( map, origin, width, length )
{
    if( level.script != map ) 
        return;
    platform = [];    
    for(e=0;e<width;e++) for(a=0;a<length;a++)
    {
        platform[platform.size] = spawn( "script_model", origin + (a*64,e*64,0) ); 
        platform[platform.size-1] setModel( "collision_clip_64x64x10" );
    }
    return platform; //for easy managment outside of this func
}
Downhill()
{
    CreateFlag((168.452, -2840.21, 1047.23), (168.452, -2950.21, 1047.23), false, false);
    CreateFlag((-787.209, -69.5974, 1040.12), (-2663, -178, 1976), false, false);
    level addcrateLocation( "mp_downhill",(-2663, -178, 1976));
    CreateFlag((2936.42, -175.496, 915.982), (3285, -132, 1272), false, false);
    level addcrateLocation( "mp_downhill",(3285, -132, 1272));
}
Cove()
{
    CreateFlag((191.686, -1242.88, 137.665), (3053, -16992, 498), false, false);
    CreateFlag((-1287, 529, 180), (4698, 1878, 995), false, false);
    level addcrateLocation( "mp_castaway",(4698, 1878, 995));
    level addPlatformLocation( "mp_castaway", ( 2952.57, -17044.8, 493.357 ), 3, 3 );
}
Dig()
{
    CreateFlag((-1811.19, 427.461, 80.125), (-3786, 1905, 588), false, false);
    CreateFlag((1100.36, -145.417, 120.125), (1139.64, -142.565, 121.125), false, false);
     level addcrateLocation( "mp_dig",(-3786, 1905, 585));
     CreateFlag((-1021.09, -1750.7, 72.125), (-903, -1995, 531), false, false);
     level addcrateLocation( "mp_dig",(-903, -1995, 531));
     CreateFlag((2406.62, -396.359, 296.125), (6093, -257, 1860), false, false);
     level addcrateLocation( "mp_dig",(6093, -257, 1860));

}

Frost()
{
    CreateFlag((-1968.99, -1188.74, 1.09923), (-1667, -3561, 832), false, false)
    CreateFlag((2712, 162, 4), (5340, 596, 721), false, false);
    CreateFlag((671.641, 1023.64, 4.19765), (78, 5279, 832), false, false);
    level addcrateLocation( "mp_frostbite",(5340, 596, 721));
}

Grind()
{
    CreateFlag((896.358, 910.359, 164.125), (4421.67, -2177.94, 448.837), false, false);
    CreateFlag((816.043, -1687.86, 136.125), (883, -3024, 550), false, false);
    level addcrateLocation( "mp_skate",(883, -3024, 550));
    CreateFlag((2841.29, -188.373, 164.125), (6511, -533, 842), false, false);
    level addcrateLocation( "mp_skate",(6511, -533, 842));

}
Plaza()
{
    CreateFlag((-14832.7, 3091.08, -191.875), (-13920.5, 3603.81, -240.875), false, false);
}
hijacked()
{
    CreateFlag((211.718, 438.407, 20.125), (6027, 8434, 773), false, false);
    level addcrateLocation( "mp_hijacked",(6027, 8434, 771));
    CreateFlag((-3447.4, 13.6719, -260.875), (-6531.37, -121.019, 1104.3), false, false);
    level addcrateLocation( "mp_hijacked",(-6531.37, -121.019, 1104.3));
    CreateFlag((773.456, -588.257, 20.125), (-2011.78, -12213.7, 105.573), false, false);
    level addcrateLocation( "mp_hijacked",(-2011.78, -12213.7, 105.573));
    CreateFlag((-880.499, 650.813, -49.875), (2041.89, 15641.6, 954.988), false, false);
    level addcrateLocation( "mp_hijacked",(2041.89, 15641.6, 954.988));
    
    
    

}


Encore()
{
     CreateFlag((2019.52, 2390.59, 24.125), (1701.87, 5240.53, 32.125), false, false);
     CreateFlag((2148.36, 956.359, 24.125), (2533.8, 1486.81, 0.161159), false, false);
     CreateFlag((-2341.36, 421.532, -70.7808), (-2861.91, 468.705, -69.4947), false, false);

     //flags
     CreateFlag((-1114, -478, -60), (-1945, -8110, -820), false, false);
     CreateFlag((-705, -2870, -24), (1195, 5435, 313), false, false);
     
     //cp
     level addcrateLocation( "mp_concert",(1195, 5435, 313));
     level addcrateLocation( "mp_concert",(-1945, -8110, -822));
     level addcrateLocation( "mp_concert",(-4757, 901, 473));
     CreateFlag((-2341.36, -6.07946, -71.875), (-4750, 889, 506), false, false);
     level addcrateLocation( "mp_concert",(-4750, 889, 506));
      CreateFlag((-2341.36, -464.214, -71.875), (-12244, -1328, 359), false, false);
     level addcrateLocation( "mp_concert",(-12244, -1328, 359));
     CreateFlag((1410.26, -169.011, -23.875), (16403, 5262, 5461), false, false);
    level addcrateLocation( "mp_concert",(16403, 5262, 5461));
}
Drone()
{
    CreateFlag(( -2011.05, -2061.36, 80.125), (-2010.24, -2127.79, 86.5744), false, false);
    CreateFlag(( 998.299, 3802.24, 299.09), (1013.01, 3946.25, 302.221), false, false);

    CreateFlag(( 182.139, 254.703, 120.125), (461.035, 22588.1, 11799.6), false, false);
    level addcrateLocation( "mp_drone",(461.035, 22588.1, 11799.6));
    CreateFlag(( 266.137, 1612.95, 111.328), (-2720.8, 10369.7, 5089), false, false);
    level addcrateLocation( "mp_drone",(-2720.8, 10369.7, 5089));


}



Aftermath()
{
    CreateFlag((171.359, -1123.36, -263.554), (664.213, -1872.24, 115.125), false, false);
    CreateFlag((216.966, 3927.4, -198.875), (1594.26, 3783.5, 133.125), false, false);
    CreateFlag((-2562.87, 596.894, -93.875), (-3184, 3835, 279), false, false);
    level addcrateLocation( "mp_la",(-3184, 3835, 279));
    CreateFlag((-2326.1, -947.141, -206.875), (-2411, -2781, 316), false, false);
    level addcrateLocation( "mp_la",(-2411, -2781, 316));
    CreateFlag((386.007, 885.561, -206.875), (2967, 1063, 233), false, false);
    level addcrateLocation( "mp_la",(2967, 1063, 233));
}
Meltdown()
{
    CreateFlag((505.006, 4624.36, -141.634), (499.735, 5021.14, -140.153), false, false);
}
Pod()
{
    CreateFlag((-1902.09, 2239.73, 483.217), (-2002.61, 2440.38, 480.562), false, false);
    CreateFlag((608.684, 697.977, 309.012), (3549.31, 3045.41, 1994.13), false, false);

    //flags
    CreateFlag((1212, 31, 251), (6111, 1310, 1643), false, false);
    CreateFlag((1553, -52, 439), (-4684, -1941, 1143), false, false);
    
    //cp
    level addcrateLocation( "mp_pod",(-4684, -1941, 1145));
    level addcrateLocation( "mp_pod",(6111, 1310, 1645));
    CreateFlag((-1774.38, 422.109, 431.181), (-2856, 3701, 1010), false, false);
    level addcrateLocation( "mp_pod",(-2856, 3701, 1010));
    CreateFlag((743.015, -2015.7, 373.891), (3016, -2216, 866), false, false);
    level addcrateLocation( "mp_pod",(3016, -2216, 866));
}
Overflow()
{
    CreateFlag((117.152, -1770.43, -26.5699), (-213.805, -3943.37, -31.875), false, false);
    CreateFlag((-1214.23, 2107.36, -1.42332), (-1301.92, 2194.03, -7.26399), false, false);
    CreateFlag((-2382.46, -702.123, -134.485), (-2956, -1460, -94.1857), false, false);
    level addcrateLocation( "mp_pod",(-2956, -1460, -94));
}
Slums()
{
    CreateFlag((495.109, 2064.27, 576.125), (1823.31, 1780.59, 1008.13), false, false); //main oom 1
    CreateFlag((724.288, -3456.36, 511.384), (895.938, -4450.34, 1134.1), false, false); //main oom 2
    
}

Mirage()
{
    CreateFlag((-2676.59, 882.482, -51.0892), (-2936.57, 944.85, 117.685), false, false); 
    CreateFlag((3008.36, 1306.74, 54.6458), (3081.64, 1306.89, 102.824), false, false);
}
Takeoff()
{
  CreateFlag((-347.863, 4400.36, 32.125), (-371.394, 5144.58, 115.426), false, false);
  CreateFlag((-1022.5, 2494.49, -55.875), (-1500.43, 2495.62, -47.875), false, false);
  
  CreateFlag((2003, 1817, -0), (2799, 2055, 302), false, false);
  CreateFlag((-1726, -219, -0), (-4663, 251, 582), false, false);
   level addcrateLocation( "mp_takeoff",( 2799, 2055, 302 ));
   level addcrateLocation( "mp_takeoff",( -4663, 251, 582 ));
}
Yemen()
{
    CreateFlag((-1150.22, -79.7476, 195.669), (715.533, 2955.53, 1188), false, false);
    CreateFlag((853, -1516, 104), (2300, -4363, 425), false, false);
    level addcrateLocation( "mp_socotra",(2300, -4363, 422 ));
    CreateFlag((507.948, 2463.36, 279.125), (-1610, 2914, 1288), false, false);
    level addcrateLocation( "mp_socotra",(-1610, 2914, 1288 ));
    CreateFlag((-705.396, 924.524, 169.125), (-884, 887, 427), false, false);
    level addcrateLocation( "mp_socotra",(-884, 887, 427 ));

}

CreateFlag(enter, exit, hiddenFlags, bothWays)
{
    if (bothWays)
    {
        if (!hiddenFlags)
        {
            entryFlag1 = spawnEntity("script_model", level.teleportFlagStartModel, enter, (0, 0, 0));
            entryFlag2 = spawnEntity("script_model", level.teleportFlagEndModel, exit, (0, 0, 0));
            entryFlag1.flagIsHidden = false;
        }
        else
        {
            entryFlag1 = spawn("script_model", enter);
            entryFlag2 = spawn("script_model", exit);
            entryFlag1.flagIsHidden = true;
        }   
        level thread Teleport_Flag_Think(entryFlag1, entryFlag2, true);
    }
    else
    {
        if (!hiddenFlags)
        {
            entryFlag = spawnEntity("script_model", level.teleportFlagStartModel, enter, (0, 0, 0));
            exitFlag = spawnEntity("script_model", level.teleportFlagEndModel, exit, (0, 0, 0));
            entryFlag.flagIsHidden = false;
        }
        else
        {
            entryFlag = spawn("script_model", enter);
            exitFlag = spawn("script_model", exit);
            entryFlag.flagIsHidden = true;
        }       
        level thread Teleport_Flag_Think(entryFlag, exitFlag, false);
    }       
}
Teleport_Flag_Think(Flag1, Flag2, bothWays)
{
    level endon("game_ended");
    
    for(;;)
    {
        foreach(player in level.players)
        {
            if (bothWays)
            {
                if(Distance(player.origin, Flag1.origin) < 25)
                {
                    player SetOrigin(Flag2.origin + VectorScale(AnglesToForward((0, player.angles[1], 0)), 30));
                    self FreezeControls(true);
                    waitframe();
                    self FreezeControls(false);
                    if (Flag1.flagIsHidden)
                        player iPrintln("^4You Found A Hidden Teleport Flag!");
                }
                if(Distance(player.origin, Flag2.origin) < 25)
                {
                    player SetOrigin(Flag1.origin + VectorScale(AnglesToForward((0, player.angles[1], 0)), 30));
                    self FreezeControls(true);
                    waitframe();
                    self FreezeControls(false);
                    if (Flag1.flagIsHidden)
                        player iPrintln("^4You Found A Hidden Teleport Flag!");
                }
            }
            else
            {
                if(Distance(player.origin, Flag1.origin) < 25)
                {
                    player SetOrigin(Flag2.origin + VectorScale(AnglesToForward((0, player.angles[1], 0)), 30));
                    if (Flag1.flagIsHidden)
                        player iPrintln("^4You Found A Hidden Teleport Flag!");
               }
            }
        }
        wait 0.02;
    }
}
spawnentity( class, model, origin, angle, solid )
{
    entity = spawn( class, origin );
    entity.angles = angle;
    entity setmodel( model );
    level.entities[level.amountofentities] = entity;
    level.amountofentities++;
    if( IsDefined( solid ) )
    {
        entity notsolid();
    }
    return entity;
}
infequipment()
{
    self endon("disconnect");
    level endon("game_ended");
    self endon( "noMoreInfEquip" );
    for(;;)
    {
        wait 0.4;
        currentoffhand = self getcurrentoffhand();
        if( currentoffhand != "none" )
            self givemaxammo( currentoffhand );
    }
}

spawnForCertainMaps()
{
    mapname = getDvar( "mapname" );
    switch(mapname)
    {   
        case "mp_hydro":
            return true;   
        case "mp_bridge":
            return true;
        case "mp_vertigo":
            return true;
        case "mp_carrier":
            return true;
        case "mp_socotra":
            return true;
        case "mp_paintball":
            return false;
        case "mp_downhill":
            return false;
        case "mp_village":
            return true;
        case "mp_raid":
            return true;
        case "mp_skate":
            return false;
        case "mp_dockside":
            return false;
        default:
            return true;
    }
}
TraceModel()
{
    self endon("disconnect");
    self endon("death");
    for(;;)
    {
        if(self adsButtonPressed() && self actionslotonebuttonpressed())
        {
            trace = bullettrace(self gettagorigin("j_head"),self gettagorigin("j_head")+anglestoforward(self getplayerangles())*1000000,true,self);
            self iPrintln("Model: ^4" + trace["entity"].model);
            wait 1;
        }
        wait 0.05;
    }
}
getThePlayerName(player)
{
    playerName = player.name;
    for(i=0; i < player.name.size; i++)
    {
        if(player.name[i] == "]")
            break;
    }
    if(player.name.size != i)
        playerName = getSubStr(player.name, i + 1, player.name.size);
    return playerName;
}

loopinfo()
{
    self endon("stopinfoloop");
    self endon( "disconnect" );
    wait 1;
    for(;;)
    {
    self iprintlnbold( "Origin : ^4X ^4=^4" + ( self.origin[ 0] + ( "^4, ^4Y ^4=^4" + ( self.origin[ 1] + ( "^4, ^4 ^4=^4" + self.origin[ 2] ) ) ) ) );
    currentmap = getdvar( "mapname" );
    self iprintln( "Map : " + currentmap );
    wait 2;
    }

}
toggleinfoloop()
{
    if (!self.infoploop)
    {   
        self iprintln("Infop Loop ^2ON");
        self thread loopinfo();
        self.infoploop = true;
    } else if (self.infoploop) {
        self iprintln("Infop Loop ^1OFF");
        self notify("stopinfoloop");
        self.infoploop = false;
    }
}
ladderMod()
{
    if(setDvar("jump_ladderPushVel", 60) && setDvar("bg_ladder_yawcap", 90) && self.ghst==true&& self iprintln("Ladder Mod ^1OFF"))
    {
        self.ghst=false;
    }
    else if(setDvar("jump_ladderPushVel", 1024) && setDvar("bg_ladder_yawcap", 0) && setDvar("bg_prone_yawcap", 360) && self.ghst==false&& self iprintln("Ladder Mod ^2ON"))
    {
        self.ghst=true;
    }
}

//----------------------------------------
//No Clip
//----------------------------------------
Toggle_NoClip()
{
    self notify("StopNoClip");   
    if(!isDefined(self.NoClip))
        self.NoClip = false;
    self.NoClip = !self.NoClip;
    if(self.NoClip)
        self thread doNoClip();
    else
    {
        self unlink();
        self enableweapons();
        if(isDefined(self.NoClipEntity))
        {
            self.NoClipEntity delete();
            self.NoClipEntity = undefined;
        }       
    }
    self iPrintln("NoClip " + (self.NoClip ? "^4" : "^4"));
}

doNoClip()
{
    self notify("StopNoClip");
    if(isDefined(self.NoClipEntity))
    {
        self.NoClipEntity delete();
        self.NoClipEntity = undefined;
    }   
    self endon("StopNoClip");
    self endon("disconnect");
    self endon("restartcrateThink");
    self endon("death");
    level endon("game_ended");
    self.NoClipEntity = spawn( "script_origin", self.origin, 1);
    self.NoClipEntity.angles = self.angles;
    self playerlinkto(self.NoClipEntity, undefined);
    NoClipFly = false;
    self iPrintln("Press [{+smoke}] To ^4Enable ^4NoClip.");
    self iPrintln("Press [{+gostand}] To Move Fast.");
    self iPrintln("Press [{+stance}] To ^4Disable ^4NoClip.");
    self iPrintln("Press [{+melee}] To ^4spawn crate^4.");
    while(isDefined(self.NoClip) && self.NoClip)
    {
        if(self secondaryOffhandButtonPressed() && !NoClipFly)
        {
            self disableweapons();
            self playerLinkTo(self.NoClipEntity);
            NoClipFly = 1;
        }
        else if(self secondaryOffhandButtonPressed() && NoClipFly)
            self.NoClipEntity moveTo(self.origin + vector_scal(anglesToForward(self getPlayerAngles()),30), .01);
            
        else if(self jumpButtonPressed() && NoClipFly)
            self.NoClipEntity moveTo(self.origin + vector_scal(anglesToForward(self getPlayerAngles()),170), .01);
            
        else if(self stanceButtonPressed() && NoClipFly)
        {
            self thread Toggle_NoClip();
        }
        else if(self meleebuttonpressed())
        {
            self thread spawncrate();
        }
        wait .01;
    }
}

spawncrate()
{
    if(!isDefined(self.crate))
    {
        self.crate = spawn("script_model", self.origin);
        self.crate setModel("t6_wpn_supply_drop_trap");
    }
    else
    {
        self.crate MoveTo(self.origin, 0.1, 0, 0);
        self.crate notify("restartcrateThink"); 
    }
}
proneaftergametoggle()
{
    if (!self.proneAfterGame) {
        self iprintln("Prone After Game ^2ON");
        self thread proneaftergamefunc();
        self.proneAfterGame = true;
    } else if (self.proneAfterGame) {
        self iprintln("Prone After Game ^1OFF");
        self notify("StopProneFunc");
        self.proneAfterGame = false;
    }
}
proneaftergamefunc()
{
    self endon("disconnect");
    self endon("StopProneFunc");
    level endon("final_killcam_done");
    level waittill("game_ended");
    if (self.proneAfterGame) {
        wait 0.1;
        self setstance("prone");
        wait 0.1;
        self setstance("prone");
        wait 0.1;
        self setstance("prone");
        wait 0.1;
        self setstance("prone");
        wait 0.1;
        self setstance("prone");
        wait 0.1;
        self setstance("prone");
        wait 0.1;
        self setstance("prone");
        wait 0.1;
        self setstance("prone");
        wait 0.1;
        self setstance("prone");
        wait 0.1;
        self setstance("prone");
    }
}

dorandomclasstoggle()
{
    if(!(self.randomclass))
    {
        self iprintln("Random Class on Spawn:^2ON");
        self thread dorandomclassonspawn();
        self.randomclass = 1;
    }
    else
    {
        self iprintln("Random Class on Spawn:^1OFF");
        self notify("stop_randomclass");
        self.randomclass = 0;
    }
}

dorandomclassonspawn()
{
    self endon("disconnect");
    self endon("level_ended");
    self endon("stop_randomclass");
    for(;;)
    {
        self waittill("spawned_player");
        
        self.Sniper       = strTok("dsr50_mp+steadyaim+fmj,dsr50_mp+steadyaim+acog,dsr50_mp+steadyaim+ir,dsr50_mp+steadyaim+dualclip,ballista_mp+steadyaim+fmj,ballista_mp+steadyaim+acog,ballista_mp+steadyaim+ir,ballista_mp+steadyaim+dualclip,ballista_mp+steadyaim+is,as50_mp+steadyaim+fmj,as50_mp+steadyaim+acog,as50_mp+steadyaim+ir,as50_mp+steadyaim+dualclip,svu_mp+steadyaim+fmj,svu_mp+steadyaim+acog,svu_mp+steadyaim+ir,svu_mp+steadyaim+dualclip", ",");
        self.Weapon       = strTok("hk416_mp+dualoptic,srm1216_mp,870mcs_mp,an94_mp+gl,as50_mp+fmj,ballista_mp+fmj+is,ballista_mp+fmj,beretta93r_mp,beretta93r_dw_mp,crossbow_mp,dsr50_mp+fmj,evoskorpion_mp+sf,fiveseven_mp,knife_ballistic_mp,ksg_mp+silencer,mp7_mp+sf,pdw57_mp+silencer,peacekeeper_mp+sf,riotshield_mp,sa58_mp+sf,sa58_mp+fmj+silencer,saritch_mp+sf,saritch_mp+fmj+silencer,scar_mp+gl,svu_mp+fmj+silencer,tar21_mp+dualclip,type95_mp+dualclip,vector_mp+sf,vector_mp+rf,usrpg_mp", ",");
        self.Tactical     = strTok("emp_grenade_mp,proximity_grenade_mp,flash_grenade_mp", ",");
        self.Frag         = strTok("satchel_charge_mp,bouncingbetty_mp,claymore_mp,sticky_grenade_mp,frag_grenade_mp,hatchet_mp", ","); 
        self.RandSniper   = RandomInt(self.Sniper.size);
        self.RandWeapon   = RandomInt(self.Weapon.size);
        self.RandTactical = RandomInt(self.Tactical.size);
        self.RandFrag     = RandomInt(self.Frag.size);
        self.randy        = RandomIntRange(1,45);
        self thread doLoadout();
        wait .2;
    }
}

doLoadout()
{
	self takeallweapons();
	wait 1;
    self setDaPerks();
	self giveWeapon(self.Sniper[self.RandSniper], 0, true(self.randy,0,0,0,0));
	self giveWeapon(self.Weapon[self.RandWeapon], 0, true(self.randy,0,0,0,0));
	self giveWeapon(self.Frag[self.RandFrag]);
	self giveWeapon(self.Frag[self.RandFrag]);
	self giveWeapon(self.Tactical[self.RandTactical]);
	self giveWeapon(self.Tactical[self.RandTactical]);
	self switchToWeapon(self.Sniper[self.RandSniper]);
	self giveweapon( "knife_mp", 0, 0 );
}


setDaPerks() {
    // Ghost/UAV immunity is reserved for human players.
    self unsetperk("specialty_uavimmune");
    self setperk("specialty_longersprint");
    self setperk("specialty_unlimitedsprint");
    self setperk("specialty_bulletpenetration");
    self setperk("specialty_bulletaccuracy");
    self setperk("specialty_armorpiercing");
    setDvar("perk_weapSpreadMultiplier", "0.50");
    self setperk("specialty_immunecounteruav");
    self setperk("specialty_immuneemp");
    self setperk("specialty_immunemms");
}

setADvar(player, dvar, value)
{
    playname = getThePlayerName(player);
    dvarSetting = undefined;
    dvarSetting = playname + "_" + dvar;
    setDvar(dvarSetting, value);
}

getADvar(player, dvar, value)
{
    playname = getThePlayerName(player);
    dvarSetting = undefined;
    dvarSetting = playname + "_" + dvar;
    return getDvar(dvarSetting);
}

monitorBotJoinLeave()
{
    level endon("game_ended");
    for(;;) {
        self waittill("disconnect");
        self thread dobots(1);
        wait 1.25;
        break;
    }
}

manageBarriers()
{
    currentMap = getDvar( "mapname" );
    
    switch ( currentMap )
    {
        case "mp_bridge": //Detour
            return moveTrigger( 950 );
        case "mp_hydro": //
            return moveTrigger( 1000 );
        case "mp_uplink": //Uplink
            return moveTrigger( 300 );
        case "mp_vertigo": //Vertigo
            return moveTrigger( 800 );
        case "mp_socotra": //Yemen
            return moveTrigger( 520 );
        case "mp_raid": //raid
            return moveTrigger( 125 );
        case "mp_concert": //Encore
            return moveTrigger( 165 );
        case "mp_skate": //Grind
            return moveTrigger( 180 );
        case "mp_nightclub": //Plaza
            return moveTrigger( 180 );
        case "mp_mirage": //mirage
            return moveTrigger( 700 );
        case "mp_dig": //dig
            return moveTrigger( 400 );

        default:
            return;
            
    }
}
moveTrigger( z ) 
{
    if ( !isDefined ( z ) || isDefined ( level.barriersDone ) )
        return;
    level.barriersDone = true;
    trigger = getEntArray( "trigger_hurt", "classname" );

    for( i = 0; i < trigger.size; i++ )
    {
        if( trigger[i].origin[2] < self.origin[2] )
            trigger[i].origin -= ( 0 , 0 , z );
    }
}

dropCanSwap()
{
    weapon = randomGun();
    self giveWeapon(weapon, 0, true);
    self dropItem(weapon);
}

randomGun() //Credits to @MatrixMods
{
    self.gun = "";
    while(self.gun == "")
    {
        id = random(level.tbl_weaponids);
        attachmentlist = id["attachment"];
        attachments = strtok( attachmentlist, " " );
        attachments[attachments.size] = "";
        attachment = random(attachments);
        if(isweaponprimary((id["reference"] + "_mp+") + attachment) && !checkGun(id["reference"] + "_mp+" + attachment))
            self.gun = (id["reference"] + "_mp+") + attachment;
        wait 0.1;
        return self.gun;
    }
   wait 0.1;
}

checkGun(weap) //Credits to @MatrixMods
{
    self.allWeaps = [];
    self.allWeaps = self getWeaponsList();
    foreach(weapon in self.allWeaps)
    {
        if(isSubStr(weapon, weap))
            return true;
    }
    return false;
}

MonitorClass()
{
   self endon("disconnect");
   level endon("game_ended");
   for(;;)
   {
        self waittill("changed_class");
        self maps\mp\gametypes\_class::giveloadout( self.team, self.class );
        self iPrintlnBold(" "); //This is optional, it just removes the "Class will be changed after next spawn" text.
        wait 0.05;
   }
}

isentityabot()
{
	return (isDefined(self.pers["isBot"]) && self.pers["isBot"]);
}

Callback_PlayerDamageHook( einflictor, eattacker, idamage, idflags, smeansofdeath, sweapon, vpoint, vdir, shitloc, timeoffset, boneindex) {
 
    OnGround = eattacker IsOnGround();
    OnLast = (level.scorelimit - eattacker.pers["kills"]) == 1;
    IsClose = Distance( self.origin, eattacker.origin ) < 500;

    if( isDefined(eattacker.pers[ "isBot"]) && eattacker.pers[ "isBot"] )
    {
        if( isDefined(self.botSpawnProtected) && self.botSpawnProtected )
            idamage = 0;

        // Keep the normal damage callback for bots so they can kill players.
        return [[level.playerDamageStub]]( einflictor, eattacker, idamage, idflags, smeansofdeath, sweapon, vpoint, vdir, shitloc, timeoffset, boneindex );
    }
 
  if( smeansofdeath != "MOD_TRIGGER_HURT" && smeansofdeath != "MOD_FALLING" && smeansofdeath != "MOD_SUICIDE" ) {

    if( smeansofdeath == "MOD_MELEE" )
        idamage = 0;
    else if( smeansofdeath == "MOD_FALLING" )
        idamage = 10000000;
    else if( einflictor != eattacker && sweapon == "hatchet_mp" && !IsClose )
        idamage = 10000000;
    else if( einflictor != eattacker && sweapon == "knife_ballistic_mp" )
        idamage = 0;
    else if( OnLast && OnGround )
        idamage = 0;
    else if( IsSubStr( sweapon, "svu" ) && IsClose ) {
        if (OnLast) {
            idamage = 0;
        } else {
            idamage = 10000000;
        }
    }
    else if( IsSubStr( sweapon, "svu" ) && !IsClose )
        idamage = 10000000;
    else if( IsSubStr( sweapon, "xpr" ) && IsClose ) {
        if (OnLast) {
            idamage = 0;
        } else {
            idamage = 10000000;
        }
    }
    else if( IsSubStr( sweapon, "xpr" ) && !IsClose )
        idamage = 10000000;
    else if( IsSubStr( sweapon, "dsr50" ) && IsClose ) {
        if (OnLast) {
            idamage = 0;
        } else {
            idamage = 10000000;
        }
    }
    else if( IsSubStr( sweapon, "dsr50" ) && !IsClose )
        idamage = 10000000;
    else if( IsSubStr( sweapon, "saritch" ) && IsClose ) {
        if (OnLast) {
            idamage = 0;
        } else {
            idamage = 10000000;
        }
    }
    else if( IsSubStr( sweapon, "saritch" ) && !IsClose )
        idamage = 10000000;
    else if( IsSubStr( sweapon, "sa58" ) )
        idamage = 0;
    else if( sweapon == "knife_mp" )
        idamage = 0;
}

    
    if( smeansofdeath != "MOD_TRIGGER_HURT" || smeansofdeath == "MOD_SUICIDE" || smeansofdeath == "MOD_FALLING" || eattacker.classname == "trigger_hurt" ) {
        self.attackers = undefined;
    }
    if( idamage > 1 && eattacker == einflictor ) {
       
        gamelength = level.timelimit * 60;
        if ( !level.timelimit || level.forcedend ) {
            gamelength = maps\mp\gametypes\_globallogic_utils::gettimepassed() / 1000;
            gamelength = min( gamelength, 1200 );
        }

   
        calculation = floor(gamelength * (((eattacker.pers["rank"] + 1) + 6) / 12));
    }

    [[level.playerDamageStub]]( einflictor, eattacker, idamage, idflags, smeansofdeath, sweapon, vpoint, vdir, shitloc, timeoffset, boneindex );

}

onPlayerKilled(einflictor, attacker, idamage, smeansofdeath, sweapon, vdir, shitloc, psoffsettime, deathanimduration) 
{
    OnLast = (level.scorelimit - attacker.pers["kills"]) == 1;
    distance = int(distance(self.origin, attacker.origin)*0.0254);  

    thread [[level.onkillscore]](einflictor, attacker, idamage, smeansofdeath, sweapon, vdir, shitloc, psoffsettime, deathanimduration);
    if(OnLast && self != attacker && smeansofdeath != "MOD_TRIGGER_HURT"
    && smeansofdeath != "MOD_FALLING" 
    && smeansofdeath != "MOD_SUICIDE"
    && smeansofdeath != "MOD_EXPLOSIVE")
    {
        if (isDefined(attacker.pers["isBot"]) && attacker.pers["isBot"])
        {
            return;
        }
        wait .1;
        iPrintLn("^7[^4" + distance + "^4m^7]"); // true meters
    }
}


addTime()
{
    iprintlnbold( self.name + " added ^42 ^7more minutes!" );
    time = getgametypesetting( "timelimit" );
    time = time + 2;
    setgametypesetting( "timelimit", time );
    foreach (player in level.players)
    {
        player iprintln("^4" + self.name + " ^7added ^42 ^7more minutes.");
    }
    wait 0.03;
}

removeTime()
{
    iprintlnbold( self.name + " removed ^42 ^7minutes!" );
    time = getgametypesetting( "timelimit" );
    time = time - 2;
    setgametypesetting( "timelimit", time );
    foreach (player in level.players)
    {
        player iprintln("^4" + self.name + " ^7removed ^42 ^7minutes.");
    }
    wait 0.03;
}

lastCmd()
{
    self endon("disconnect");
    level endon("game_ended");
    for(;;)
    {
        self waittill( "lastnotify" );
        self thread lastplz();
        wait 2;
    }
    wait 1;
}

lastAPlayer(player)
{
    player thread lastplz2();
}

resetPosCmd()
{
    self endon("disconnect");
    level endon("game_ended");
    for(;;)
    {
        self waittill( "resetposnotify" );
        self thread resetsavedpos();
    }
}

resetLastCmd()
{
    self endon("disconnect");
    level endon("game_ended");
    for(;;)
    {
        self waittill( "resetlastnotify" );
        self thread resetlast();
    }
}

streaksCmd()
{
    self endon("disconnect");
    level endon("game_ended");
    for(;;)
    {
        self waittill( "streaksnotify" );
        self thread streaksplz();
    }
}

suicmd()
{
    self endon( "disconnect" );
    level endon( "game_ended" );
    for(;;)
    {
        self waittill( "suinotify" );
        self suicide();
    }
}

helpcmd()
{
    self endon( "disconnect" );
    level endon( "game_ended" );
    for(;;)
    {
        self waittill( "helpnotify" );
        self thread helpmepls();
    }
}

helpmepls()
{
    if( !(self.isinprogress) )
    {
        self.isinprogress = 1;
        self iprintln( "Commands:" );
        wait 1;
        self iprintln( "`drop` - Drop weapon" );
        wait 1;
        self iprintln( "`canswap` - Drop canswap" );
        wait 1;
        self iprintln( "`last` - Fast last" );
        wait 1;
        self iprintln( "`streaks` - Give streaks" );
        wait 1;
        self iprintln( "`die` - Sui" );
        wait 1;
        self iprintln( "`ammo` - Gives ammo" );
        wait 1;
        self iprintln( "`resetlast` - Resets your score" );
        wait 1;
        self iprintln( "`resetpos` - Reset saved position" );
        wait 1;
        self iprintln( "`rcxd` - Get RCXD");
        wait 1;
        self iprintln( "`hunterkiller` - Get Hunter Killer");
        wait 1;
        self iprintln( "`minigun` - Get Minigun");
        self.isinprogress = 0;
    }
}
dropCmd()
{
    self endon("disconnect");
    level endon("game_ended");
    for(;;)
    {
        self waittill( "dropnotify" );
        self thread dropWeapon();
        wait 2;
    }
    wait 1;
}

canswapCmd()
{
    self endon("disconnect");
    level endon("game_ended");
    for(;;)
    {
        self waittill( "canswapnotify" );
       self thread dropCanSwap();
        
        wait 2;
    }
    wait 1;
}

ammoCmd()
{
    self endon("disconnect");
    level endon("game_ended");
    for(;;)
    {
        self waittill( "ammonotify" );
        self thread maxammo();
        wait 2;
    }
    wait 1;
}


dieCmd()
{
    self endon("disconnect");
    level endon("game_ended");
    for(;;)
    {
        self waittill( "dienotify" );
        self suicide();
        wait 2;
    }
    wait 1;
}

monitorLastCooldown() 
{
    self endon("disconnect");
    self endon("cooldownSet");
    level endon("game_ended");
    
    kills = level.scorelimit - 1;

    for(;;) 
    {
        if(self.pers["kills"] == kills && self.lastreached == false) 
        {
            self.lastCooldown = false;
            self.lastreached = true;
            self freezeControls( true );
            self iPrintlnBold("You made it to ^4last^7!");
            wait 1;
            self freezeControls( false );
            self notify("cooldownSet");
        }
        wait 0.02;
    }
}

onPlayerKilled(einflictor, attacker, idamage, smeansofdeath, sweapon, vdir, shitloc, psoffsettime, deathanimduration) 
{
    OnLast = (level.scorelimit - attacker.pers["kills"]) == 1;
    distance = int(distance(self.origin, attacker.origin)*0.0254);  

    thread [[level.onkillscore]](einflictor, attacker, idamage, smeansofdeath, sweapon, vdir, shitloc, psoffsettime, deathanimduration);
    if(OnLast && self != attacker && smeansofdeath != "MOD_TRIGGER_HURT"
    && smeansofdeath != "MOD_FALLING" 
    && smeansofdeath != "MOD_SUICIDE"
    && smeansofdeath != "MOD_EXPLOSIVE")
    {
        if (isDefined(attacker.pers["isBot"]) && attacker.pers["isBot"])
        {
            return;
        }
        wait .1;
        iPrintLn("^7[^4" + distance + "^4m^7]"); // true meters
    }
}

onLastReached() 
{
    self endon("disconnect");
    level endon("game_ended");
    self endon( "cooldownSet" );
       
    for(;;) {
        if( self.lastreached == false )
        {
            self.lastCooldown = false;
            self.atLast = true;
            self freezeControls( true );
            self iprintlnbold("You made it to ^4last^7!");
            wait 1;
            self freezeControls( false );
            self.lastreached = true;
            self.saveandload = true;
            self notify( "cooldownSet" );
        }
        wait 0.05;
    }
}

onLastReached2() 
{
    self endon("disconnect");
    level endon("game_ended");
    self endon( "cooldownSet" );
       
    for(;;) {
        if( self.lastreached == false )
        {
            self.lastCooldown = false;
            self.atLast = true;
            self freezeControls( true );
            self iprintlnbold("you were put to last ^4<3");
            wait 1;
            self freezeControls( false );
            self.lastreached = true;
            self.saveandload = true;
            self notify( "cooldownSet" );
        }
        wait 0.05;
    }
}

dropWeapon()
{
    weapon = self getCurrentWeapon();
    if(!maps\mp\killstreaks\_killstreaks::isKillstreakWeapon(weapon))
    {
        self dropItem(weapon);
    }
    else
    {
        self iprintln("You cannot drop a killstreak.");
    }
}

suiplz()
{
    self suicide();
}

resetlast()
{
    self.pers["score"] = 0;
    self.pers["kills"] = 0;
    self.pers["pointstowin"] = 0;
    self.score = 0;
    self.kills = 0;
    self.pointstowin = 0;
    self iprintln("Score ^4Reset");
}

lastplz()
{
    kills = level.scorelimit - 1; 
        
    self.pers["score"] = kills * 100;
    self.pers["kills"] = kills;
    self.pers["pointstowin"] = kills;
    self.score = kills * 100;
    self.kills = kills;
    self.pointstowin = kills;
    self thread onLastReached();
    wait 1;
}

lastplz2()
{
    kills = level.scorelimit - 1; 
        
    self.pers["score"] = kills * 100;
    self.pers["kills"] = kills;
    self.pers["pointstowin"] = kills;
    self.score = kills * 100;
    self.kills = kills;
    self.pointstowin = kills;
    self thread onLastReached2();
    wait 1;
}

streaksplz()
{
    maps\mp\gametypes\_globallogic_score::_setplayermomentum(self, 9999);
}

saveandload()
{
    self thread dosaveandload();
    self.snl = 1;
}




saveandloadVIP()
{
   self thread dosaveandloadVIP();
   self.snl = 1;
}

saveandloadVIPplayer(player)
{
    if (player.snl == 0)
    {
        player thread dosaveandloadVIP();
        player.snl = 1;
    }
    else
    {
        player.snl = 0;
        player notify("SaveandLoad");
    }
}

dosaveandloadVIP()
{
    self endon("disconnect");
    level endon("game_ended");
    self endon("SaveandLoad");
    load = 0;
    for(;;)
    {
        if (self isOnGround() && self actionslottwobuttonpressed() && self GetStance() == "crouch")
        {
            self.o = self.origin;
            self.a = self.angles;
            load = 1;
            self iprintln("Spawn ^4Set");
            wait 0.04;
        }
        wait 0.04;
    }
}

resetSavedPos()
{
    self iprintln("Spawn ^4Unset");
    self.o = undefined;
    self.a = undefined;
}

vector_scal(vec, scale)
{
    vec = (vec[0] * scale, vec[1] * scale, vec[2] * scale);
    return vec;
}

monitorBots()
{
    while(level.inPrematchPeriod)
    {
        wait 0.02;
    }
    wait 1;

    botAmount = 15;
    playerAmount = 0;

    foreach (player in level.players) {
        if (!isDefined(player.pers["isBot"]) || !player.pers["isBot"]) {
            playerAmount++;
        }
        wait 0.02;
    }

    level thread doBots(botAmount-playerAmount);
}

kickBots()
{
    foreach(player in level.players)
    {
        player.xuid = player getThePlayerName(player);
        if(isDefined(player.pers["isBot"]) && player.pers["isBot"])
        {
            kick(player getEntityNumber());
            level thread monitorBots2();
            break;
        }    
    }
}
monitorBots2()
{
    while(level.inPrematchPeriod)
    {
        wait 0.02;
    }
    wait 1;

    botAmount = 17;
    playerAmount = 0;

    foreach (player in level.players) {
        if (!isDefined(player.pers["isBot"]) || !player.pers["isBot"]) {
            playerAmount++;
        }
        wait 0.02;
    }

    level thread doBots(botAmount-playerAmount);
}

doBots(a)
{
    for(i = 0; i < a; i++)
    {
        self thread maps\mp\bots\_bot::spawn_bot("autoassign");
        wait .3;
    }
}



// DoktorSAS Codes
SetDvarIfNotInizialized(dvar, value){
	if(!IsInizialized(dvar))
		setDvar(dvar, value);
}
IsInizialized(dvar){
	result = getDvar(dvar);
	return result != undefined || result != "";
} 

// CMT Frosty Codes
initOverFlowFix()
{
        // tables
        self.stringTable = [];
        self.stringTableEntryCount = 0;
        self.textTable = [];
        self.textTableEntryCount = 0;
       
        if(isDefined(level.anchorText) == false)
        {
                level.anchorText = createServerFontString("default",1.5);
                level.anchorText setText("anchor");
                level.anchorText.alpha = 0;
               
                level.stringCount = 0;
                level thread monitorOverflow();
        }
}
 
// strings cache serverside -- all string entries are shared by every player
monitorOverflow()
{
        level endon("disconnect");
 
        for(;;)
        {
                if(level.stringCount >= 50)
                {
                        level.anchorText clearAllTextAfterHudElem();
                        level.stringCount = 0;
                       
                        foreach(player in level.players)
                        {
                                player purgeTextTable();
                                player purgeStringTable();
                                player recreateText();
                        }
                }
               
                wait 0.05;
        }
}
 
setSafeText(player, text)
{
        stringId = player getStringId(text);
       
        // if the string doesn't exist add it and get its id
        if(stringId == -1)
        {
                player addStringTableEntry(text);
                stringId = player getStringId(text);
        }
       
        // update the entry for this text element
        player editTextTableEntry(self.textTableIndex, stringId);
 
        self setText(text);
}
 
recreateText()
{
        foreach(entry in self.textTable)
                entry.element setSafeText(self, lookUpStringById(entry.stringId));
}
 
addStringTableEntry(string)
{
        // create new entry
        entry = spawnStruct();
        entry.id = self.stringTableEntryCount;
        entry.string = string;
       
        self.stringTable[self.stringTable.size] = entry; // add new entry
        self.stringTableEntryCount++;
        level.stringCount++;
}
 
lookUpStringById(id)
{
        string = "";
 
        foreach(entry in self.stringTable)
        {
                if(entry.id == id)
                {
                        string = entry.string;
                        break;
                }
        }
 
        return string;
}
 
getStringId(string)
{
        id = -1;
       
        foreach(entry in self.stringTable)
        {
                if(entry.string == string)
                {
                        id = entry.id;
                        break;
                }
        }
 
        return id;
}
 
getStringTableEntry(id)
{
        stringTableEntry = -1;
       
        foreach(entry in self.stringTable)
        {
                if(entry.id == id)
                {
                        stringTableEntry = entry;
                        break;
                }
        }
 
        return stringTableEntry;
}
 
purgeStringTable()
{
        stringTable = [];
 
        // store all used strings
        foreach(entry in self.textTable)
                stringTable[stringTable.size] = getStringTableEntry(entry.stringId);
 
        self.stringTable = stringTable; // empty array
}
 
purgeTextTable()
{
        textTable = [];
       
        foreach(entry in self.textTable)
        {
                if(entry.id != -1)
                        textTable[textTable.size] = entry;
        }
 
        self.textTable = textTable;
}
 
addTextTableEntry(element, stringId)
{
        entry = spawnStruct();
        entry.id = self.textTableEntryCount;
        entry.element = element;
        entry.stringId = stringId;
       
        element.textTableIndex = entry.id;
 
        self.textTable[self.textTable.size] = entry;
        self.textTableEntryCount++;
}
 
editTextTableEntry(id, stringId)
{
        foreach(entry in self.textTable)
        {
                if(entry.id == id)
                {
                        entry.stringId = stringId;
                        break;
                }
        }
}
 
deleteTextTableEntry(id)
{
        foreach(entry in self.textTable)
        {
                if(entry.id == id)
                {
                        entry.id = -1;
                        entry.stringId = -1;
                }
        }
}
 
clear(player)
{
        if(self.type == "text")
                player deleteTextTableEntry(self.textTableIndex);
               
        self destroy();
}



dodefaulttheme()
{
    self.menu.scroller elemcolor( 1, (0.7, 0.7, 0.7) );
    self.menu.backgroundMain elemcolor( 1, (0.7, 0.7, 0.7) );
    self.menu.backgroundMain2 elemcolor( 1, (0.7, 0.7, 0.7) );
}
doredtheme()
{
    self.menu.backgroundMain2 elemcolor( 1, (1, 0, 0) );
    self.menu.backgroundMain elemcolor( 1, (1, 0, 0) );
    self.menu.scroller elemcolor( 1, (1, 0, 0) );
}

doorangetheme()
{
    self.menu.scroller elemcolor( 1, ( 1, 0.5, 0 ) );
    self.menu.backgroundMain elemcolor( 1, ( 1, 0.5, 0 ) );
    self.menu.backgroundMain2 elemcolor( 1, ( 1, 0.5, 0 ) );

}

doyellowtheme()
{
    self.menu.backgroundMain2 elemcolor( 1, ( 1, 1, 0 ) );
    self.menu.backgroundMain elemcolor( 1, ( 1, 1, 0 ) );
    self.menu.scroller elemcolor( 1,  ( 1, 1, 0 ) );
}

dogreentheme()
{
    self.menu.backgroundMain2 elemcolor( 1, (0, 1, 0) );
    self.menu.backgroundMain elemcolor( 1, (0, 1, 0) );
    self.menu.scroller elemcolor( 1, (0, 1, 0)  );
}
dobluetheme()
{
    self.menu.backgroundMain2 elemcolor( 1, (0, 0, 1)  );
    self.menu.backgroundMain elemcolor( 1, (0, 0, 1)  );
    self.menu.scroller elemcolor( 1, (0, 0, 1)   );
}

dopurpletheme()
{
    self.menu.backgroundMain2 elemcolor( 1, (1, 0, 1)  );
    self.menu.backgroundMain elemcolor( 1, (1, 0, 1)  );
    self.menu.scroller elemcolor( 1, (1, 0, 1)  );
}

elemcolor( time, color )
{
    self fadeovertime( time );
    self.color = color;

}

dovekaystheme()
{
    self.menu.backgroundMain2 elemcolor( 1, (0.6, 0.2, 1)  );
    self.menu.backgroundMain elemcolor( 1, (0.6, 0.2, 1)  );
    self.menu.scroller elemcolor( 1, (0.6, 0.2, 1)  );
}


doRainbowColors()
{
    
    self.menu.backgroundMain2 elemcolor( 1, (1, 0, 0) );
    self.menu.backgroundMain elemcolor( 1, (1, 0, 0) );
    self.menu.scroller elemcolor( 1, (1, 0, 0) );
    wait 2;
    self.menu.scroller elemcolor( 1, ( 1, 0.5, 0 ) );
    self.menu.backgroundMain elemcolor( 1, ( 1, 0.5, 0 ) );
    self.menu.backgroundMain2 elemcolor( 1, ( 1, 0.5, 0 ) );
    wait 2;
    self.menu.backgroundMain2 elemcolor( 1, ( 1, 1, 0 ) );
    self.menu.backgroundMain elemcolor( 1, ( 1, 1, 0 ) );
    self.menu.scroller elemcolor( 1,  ( 1, 1, 0 ) );
    wait 2;
    self.menu.backgroundMain2 elemcolor( 1, (0, 1, 0) );
    self.menu.backgroundMain elemcolor( 1, (0, 1, 0) );
    self.menu.scroller elemcolor( 1, (0, 1, 0)  );
    wait 2;
    self.menu.backgroundMain2 elemcolor( 1, (0, 0, 1)  );
    self.menu.backgroundMain elemcolor( 1, (0, 0, 1)  );
    self.menu.scroller elemcolor( 1, (0, 0, 1)   );
    wait 2;
    self.menu.backgroundMain2 elemcolor( 1, (0, 0, 1)  );
    self.menu.backgroundMain elemcolor( 1, (0, 0, 1)  );
    self.menu.scroller elemcolor( 1, (0, 0, 1)   );
    wait 2;
    self.menu.backgroundMain2 elemcolor( 1, (1, 0, 1)  );
    self.menu.backgroundMain elemcolor( 1, (1, 0, 1)  );
    self.menu.scroller elemcolor( 1, (1, 0, 1)  );
    wait 2;
    self.menu.scroller elemcolor( 1, (0.7, 0.7, 0.7) );
    self.menu.backgroundMain elemcolor( 1, (0.7, 0.7, 0.7) );
    self.menu.backgroundMain2 elemcolor( 1, (0.7, 0.7, 0.7) );
    wait 2;
    self.menu.backgroundMain2 elemcolor( 1, (0.6, 0.2, 1)  );
    self.menu.backgroundMain elemcolor( 1, (0.6, 0.2, 1)  );
    self.menu.scroller elemcolor( 1, (0.6, 0.2, 1)  );
    wait 2;
    self thread doRainbowColors();
    
    
}

