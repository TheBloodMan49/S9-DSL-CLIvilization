grammar CLIvilization

entry Model:
    sections+=Section* ;

Section:
    Size | Cities | Game | VictoryConditions | BuildingDefArray | UnitDefArray
;

Size:
    '[size]'
    'x' '=' x=INT 
    'y' '=' y=INT
    
;

Cities:
    '[cities]' cities+=City (cities+=City)*;

City:
    name=Value '{'
        'x' '=' x=INT
        'y' '=' y=INT
        'color' '=' color=COLOR
        'starting_resources' '=' startingResources=INT
        'player_type' '=' playerType=PlayerType
        'nb_slots_buildings' '=' nbSlotsBuildings=INT
        'starting_buildings' '=' buildings=BuildingInstanceArray
        ('blacklist_buildings' '=' blacklist_buildings=ValueArray)?
        ('whitelist_buildings' '=' whitelist_buildings=ValueArray)?
        'nb_slots_units' '=' nbSlotsUnits=INT
        'starting_units' '=' units=UnitInstanceArray
        ('blacklist_units' '=' blacklist_units=ValueArray)?
        ('whitelist_units' '=' whitelist_units=ValueArray)?
    '}'
;

PlayerType returns string:
    'PLAYER' | 'AI';

BuildingInstanceArray:
    '[' (elements+=BuildingInstance (',' elements+=BuildingInstance)*)? ']'
;

BuildingInstance:
    '{'
        'id_building' '=' id_building=Value
        'level' '=' level=INT
    '}'
;

UnitInstanceArray:
    '[' (units+=UnitInstance (',' units+=UnitInstance)*)? ']'
;

UnitInstance:
    '{'
        'id_units' '=' id_units=Value
        'nb_units' '=' nb_units=INT
    '}'
;

Game:
    '[game]'
    'current_turn' '=' currentTurn=INT
    'ui_color' '=' uiColor=COLOR
;

VictoryConditions:
    '[victory_conditions]'
    'nb_turns' '=' nbTurns=INT
    'resources_spent' '=' resourcesSpent=INT
;

BuildingDefArray:
    '[buildings]' (buildings+=BuildingDef (buildings+=BuildingDef)*)?;

BuildingDef:
    name=Value '{'
        'cost' '=' cost=INT
        'build_time' '=' buildTime=INT
        'slots' '=' slots=INT
        'production' '=' production=Production
        'prerequisites' '=' prerequisites=PrereqArray
    '}'
;

Production:
    '{'
        'type' '=' prodType=ProductionType
        ('id_units' '=' prodUnitId=Value)?
        'amount' '=' amount=INT
        'time' '=' time=INT
        'cost' '=' cost=INT
    '}'
;

ProductionType returns string:
    "unit" | "ressource" ;

PrereqArray:
    '[' (prereqs+=Prereq (',' prereqs+=Prereq)*)? ']'
;

Prereq:
    '{' 'id_building' '=' id_building=Value '}'
;

UnitDefArray:
    '[units]' (units+=UnitDef (units+=UnitDef)*)?;

UnitDef:
    name=Value '{'
        'attack' '=' attack=INT
    '}'
;

Value returns string:
    STRING | ID;

/* ---------------------------
   Terminals / whitespace / comments
   --------------------------- */

terminal COLOR:
    /#[0-9a-fA-F]{6}/
;

terminal ID:
    ( 'a'..'z' | 'A'..'Z' | 'À'..'ÿ' | '_' )
    ( 'a'..'z' | 'A'..'Z' | 'À'..'ÿ' | '0'..'9' | '_' )*
;

IntArray:
    '[' (values+=INT (',' values+=INT)*)? ']'
;

ValueArray:
    '[' (values+=Value (',' values+=Value)*)? ']'
;

terminal INT returns number:
    ('0'..'9')+
;

terminal STRING:
    '"' ( '\\' . | !('\\'|'"') )* '"'
;

hidden terminal ML_COMMENT:
    '/\*' -> '*\/'
;

hidden terminal SL_COMMENT:
    '//' !('\n' | '\r')* ('\r'? '\n')?
;

hidden terminal WS:
    (' '|'\t'|'\r'|'\n')+
;
