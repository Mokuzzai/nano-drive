extends Node2D

var engine: EngineHandle

func _ready() -> void:
	engine = EngineHandle.create_new(
		"res://../target/debug/nd-engine.exe"
	)
