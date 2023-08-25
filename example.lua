local yield = coroutine.yield

local function onEvent()
	yield(say("Hello!"))
	say("I'm moving")
	move(0, 2)
	-- Explicit yield type.
	-- You only need this when multiple events have been started and you only want to wait for one.
	yield(Event.WaitMove)
	-- Alternatively, wait for both to complete:
	yield({
		say("Moving again!"),
		move(2, 0),
	})
end

event = coroutine.create(onEvent)
