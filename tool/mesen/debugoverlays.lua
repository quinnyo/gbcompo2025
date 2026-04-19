
local function fmt_bin(x, w)
	w = w or 1
	local out = ""
	while x > 0 or w > 0 do
		if x & 1 == 1 then
			out = "1" .. out
		else
			out = "0" .. out
		end
		x = x >> 1
		w = w - 1
	end
	return out
end



-------- Overlay guy is meant to help layout I guess --------

local function createOverlay(margin, scale)
	local obj = {
		scale = scale or 2,
		margin = margin or 8,
		surface = emu.drawSurface.scriptHud,
	}
	obj.fg = 0xE7DFC9
	obj.bg = 0x2D1B33
	obj.coleru = {
		0xBF2A7F, 0xE84D5B, 0xF4A854, 0x9A947C, 0xA14016, 0x142026, 0xFF8826, 0xA6E094,
		0xE7EDEA, 0xFFF8BC, 0x89A194, 0xCC883A, 0xF03813, 0x5B756C, 0xCCAC95, 0x26979F,
		0xE4E391, 0xEE887A, 0x2C9FA3, 0xF07360, 0x20130A, 0x030D4F, 0x322938, 0xF3214E,
		0xFFC52C, 0x5C3D5B, 0x9ABC8A, 0xEAE2CF, 0x123142, 0x748B83, 0xE9F0C9, 0xE8E490,
		0x2D1B33, 0xFB0C06, 0xF36A71, 0x9D7E79, 0x3B3B3B, 0xB4CCB9, 0x3B657A, 0xCEECEF,
		0xCFC89A, 0xB31237, 0xCF023B, 0xFFB914, 0xFF703F, 0x76BCAD, 0xFA2E59, 0x5C4152,
	}
	obj.select = function(self)
		emu.selectDrawSurface(self.surface, self.scale)
	end
	obj.getMargin = function(self)
		local m = {
			x0 = 2 * self.scale,
			y0 = 4 * self.scale,
			x1 = 2 * self.scale,
			y1 = 4 * self.scale,
		}
		if type(self.margin) == "table" then
			m.x0 = self.margin.x0 or self.margin.x or m.x0
			m.y0 = self.margin.y0 or self.margin.y or m.y0
			m.x1 = self.margin.x1 or self.margin.x or m.x1
			m.y1 = self.margin.y1 or self.margin.y or m.y1
		elseif type(self.margin) == "number" then
			m.x0 = self.margin
			m.y0 = self.margin
			m.x1 = self.margin
			m.y1 = self.margin
		end
		return m
	end
	obj.getOrigin = function(self)
		local m = self:getMargin()
		return m.x0, m.y0
	end
	obj.getEnd = function(self)
		local m = self:getMargin()
		local surfW, surfH = self:getSurfaceSize()
		return surfW - m.x1, surfH - m.y1
	end
	obj.getSize = function(self)
		local x0, y0 = self:getOrigin()
		local x1, y1 = self:getEnd()
		return x1 - x0, y1 - y0
	end
	obj.getSurfaceSize = function(self)
		return 160 * self.scale, 144 * self.scale
	end
	obj.oamToOverlay = function(self, x, y)
		local ox, oy = self:getOrigin()
		return self.scale * x - ox, self.scale * y - oy
	end
	obj.getLocalMouse = function(self)
		local mouse = emu.getMouseState()
		local x,y = self:oamToOverlay(mouse.x, mouse.y)
		mouse.localX = x
		mouse.localY = y
		return mouse
	end
	obj.drawLine = function(self, x, y, x2, y2, color, duration, delay)
		self:select()
		local ox, oy = self:getOrigin()
		emu.drawLine(ox + x, oy + y, ox + x2, oy + y2, color, duration, delay)
	end
	obj.drawPixel = function(self, x, y, color, duration, delay)
		self:select()
		local ox, oy = self:getOrigin()
		emu.drawPixel(ox + x, oy + y, color, duration, delay)
	end
	obj.drawRectangle = function(self, x, y, width, height, color, fill, duration, delay)
		self:select()
		local ox, oy = self:getOrigin()
		emu.drawRectangle(ox + x, oy + y, width, height, color, fill, duration, delay)
	end
	obj.drawString = function(self, x, y, text, textColor, backgroundColor, maxWidth, duration, delay)
		self:select()
		local ox, oy = self:getOrigin()
		emu.drawString(ox + x, oy + y, text, textColor or self.fg, backgroundColor or self.bg, maxWidth, duration, delay)
	end

	obj.drawLine2 = function(self, x0, y0, x1, y1, color, shadowColor)
		shadowColor = shadowColor or 0
		self:drawLine(x0 + 1, y0 + 1, x1 + 1, y1 + 1, shadowColor)
		self:drawLine(x0, y0, x1, y1, color)
	end

	obj.drawRectangle2 = function(self, x, y, width, height, color, fillColor)
		if not fillColor then
			local a = (color << 2) & 0xFF000000
			fillColor = a | (color & 0x00FFFFFF)
		end
		self:select()
		local ox, oy = self:getOrigin()
		emu.drawRectangle(ox + x, oy + y, width, height, fillColor, true)
		emu.drawRectangle(ox + x, oy + y, width, height, color, false)
	end

	obj.drawVector = function(self, cx, cy, vx, vy, magMax, displayMag, color, bgcolor)
		color = color or 0xFF00FF
		local vecScale = displayMag / magMax
		local w = displayMag * 2
		self:drawRectangle(cx - displayMag, cy - displayMag, w, w, bgcolor or 0xA0000000, true)
		self:drawRectangle(cx - displayMag, cy - displayMag, w, w, color | 0x80000000)
		self:drawLine(cx, cy, cx + vx * vecScale, cy + vy * vecScale, color)
		self:drawLine(cx, 1 + cy, cx + vx * vecScale, 1 + cy + vy * vecScale, color)
		self:drawLine(1 + cx, cy, 1 + cx + vx * vecScale, cy + vy * vecScale, color)
	end

	obj.drawWindow = function(self)
		local w, h = self:getSize()
		self:drawRectangle(0, 0, w, h, 0x0F9BABEB, false)
	end

	obj.drawTilemap = function(self, originx, originy, tiles, mapw, maph, tile_size)
		local boundw = mapw * tile_size
		local boundh = maph * tile_size
		self:drawRectangle(originx, originy, boundw, boundh, 0x20606060, true)
		self:drawRectangle(originx, originy, boundw, boundh, 0x10606060, false)
		if #tiles ~= mapw * maph then
			self:drawString(string.format("#tiles(%d) ~= %d", #tiles, mapw*maph))
			return
		end
		for y = 0, maph - 1 do
			local py = originy + y * tile_size
			for x = 0, mapw - 1 do
				local px = originx + x * tile_size
				local c = tiles[1 + y * mapw + x]
				if c > 0 and c <= #self.coleru then
					self:drawRectangle(px, py, tile_size, tile_size, self.coleru[c], true)
				elseif c ~= 0 then
					self:drawString(px, py, tostring(c))
				end
			end
		end
	end

	obj.drawTable = function(self, originx, originy, t)
		local dumpTable = function(t, depth, lines)
			depth = depth or 0
			lines = lines or {}
			local keys = {}
			for k,_ in pairs(t) do
				table.insert(keys, k)
			end
			table.sort(keys)
			for _i,k in ipairs(keys) do
				local v = t[k]
				if type(v) == "table" then
					table.insert(lines, { depth = depth, str = k..":" })
					dumpTable(v, depth + 1, lines)
				else
					table.insert(lines, { depth = depth, str = k..": "..tostring(v) })
				end
			end
			return lines
		end
		local x = originx
		local y = originy
		for _i,line in ipairs(dumpTable(t)) do
			local sz = emu.measureString(line.str)
			self:drawString(x + line.depth * 4, y, line.str)
			y = y + sz.height
		end
	end

	return obj
end


-------- Coord utility --------

local Coord = {
	MAX_UNITS = 4096,
}
Coord.units = function(c)
	return c >> 4
end
Coord.subs = function(c)
	return c & 0x0F
end
Coord.frac = function(c)
	return Coord.subs(c) / 16.0
end
Coord.ctof = function(c)
	return Coord.units(c) + Coord.frac(c)
end
Coord.fmt = function(c)
	return string.format("$%04X.%X", c >> 4, c & 0x0F)
end



-------- st: struct helper thing --------

local function st_field(t, name, size, signed)
	if t._ofs == nil then
		t._ofs = 0
	end
	local field = {}
	field.name = name
	field.ofs = t._ofs
	field.size = size
	field.signed = signed or false
	table.insert(t, field)
	t[name] = field
	t._ofs = t._ofs + size
	t.size = t._ofs
end


local function st_readFields(addr, memType, spec)
	local data = {}
	for _, field in ipairs(spec) do
		if field.size == 1 then
			data[field.name] = emu.read(addr + field.ofs, memType, field.signed)
		elseif field.size == 2 then
			data[field.name] = emu.read16(addr + field.ofs, memType, field.signed)
		elseif field.size == 4 then
			data[field.name] = emu.read32(addr + field.ofs, memType, field.signed)
		end
	end
	return data
end


local function st_read(st, addr, memType, ident)
	local data = {}
	data._st = st
	data._addr = addr
	data._memType = memType
	data._ident = ident
	for _, field in ipairs(st) do
		if field.size == 1 then
			data[field.name] = emu.read(addr + field.ofs, memType, field.signed)
		elseif field.size == 2 then
			data[field.name] = emu.read16(addr + field.ofs, memType, field.signed)
		elseif field.size == 4 then
			data[field.name] = emu.read32(addr + field.ofs, memType, field.signed)
		end
	end
	return data
end


local function st_monitor(st, instAddr, memType, ident)
	local mon = {}
	mon._st = st
	mon.state = st_read(st, instAddr, memType, ident or "(mon)")
	mon.changes = {}
	mon.tick = function(self)
		for _,field in ipairs(st) do
			self.changes[field.name] = {}
		end
	end
	mon.write = function(self, ifield, address, value)
		local field = self._st[ifield]
		self.state[field.name] = value
		table.insert(self.changes[field.name], value)
		if self.onWrite and type(self.onWrite) == "function" then
			self.onWrite(ifield, address, value)
		end
	end
	for i,field in ipairs(st) do
		mon.changes[field.name] = {}
		local fieldAddr = instAddr + field.ofs
		if field.size == 1 then
			emu.addMemoryCallback(function(address, value)
				if field.signed then
					value = value < 0x80 and value or value - 256
				end
				mon:write(i, address, value)
			end, emu.callbackType.write, fieldAddr, fieldAddr, emu.cpuType.gameboy, memType)
		else
			emu.addMemoryCallback(function(address, value)
				local fieldRelAddr = address - fieldAddr
				local state = mon.state[field.name]
				local shiftBits = fieldRelAddr * 8
				local mask = 0xFF << shiftBits
				local newValue = (value << shiftBits) | (state & ~mask)
				mon:write(i, address, newValue)
			end, emu.callbackType.write, fieldAddr, fieldAddr + field.size - 1)
		end
	end
	return mon
end


local function st_create(name)
	local st = {}
	st.name = name or "Struct"

	-- Same as emu.getLabelAddress but try looking up the first field if
	-- symbol isn't found.
	-- Necessary because: for a given address, Mesen only keeps the last
	-- label encountered in the .sym file. So if `wLabel::` is at the same
	-- address as `wLabel.x:: db`, whichever appears last in the .sym file
	-- will be the only one that Mesen knows about.
	st.getLabelAddress = function(self, symbol)
		local label = emu.getLabelAddress(symbol)
		if not label and #self > 0 then
			local fieldSym = symbol .. "_" .. self[1].name
			label = emu.getLabelAddress(fieldSym)
		end
		return label
	end

	st.readFromLabel = function(self, symbol, offset)
		local label = self:getLabelAddress(symbol)
		if not label then
			return
		end
		return st_read(self, label.address + (offset or 0), label.memType)
	end

	return st
end


local function st_fmt_field(fmt, x)
	if type(fmt) == "string" then
		return string.format(fmt, x)
	elseif type(fmt) == "function" then
		return fmt(x)
	else
		return nil
	end
end


local function st_fmt(obj, ident)
	local st = obj._st
	ident = ident or string.format("%s %s @$%04X", st.name, obj._ident or "-", obj._addr)
	local out = ident
	if st.fieldfmt then
		out = out .. ":\n"
		for _, field in ipairs(st) do
			local s = st_fmt_field(st.fieldfmt[field.name], obj[field.name])
			if s then
				out = out .. string.format(" .%s: %s\n", field.name, s)
			end
		end
	end
	return out
end



-------- Entity --------

Entity = st_create("Entity")
st_field(Entity, "Info", 1)
st_field(Entity, "Ctrl", 1)
st_field(Entity, "AccX", 1, true)
st_field(Entity, "VelX", 1, true)
st_field(Entity, "PosX", 2)
st_field(Entity, "AccY", 1, true)
st_field(Entity, "VelY", 1, true)
st_field(Entity, "PosY", 2)
st_field(Entity, "Collide", 2)
st_field(Entity, "DispX", 1)
st_field(Entity, "DispY", 1)
st_field(Entity, "_PAD_", 2)

Entity.fieldfmt = {
	Info = "$%02X",
	Ctrl = function(x)
		return fmt_bin(x, 8)
	end,
	AccX = "%3d",
	VelX = "%3d",
	PosX = Coord.fmt,
	AccY = "%3d",
	VelY = "%3d",
	PosY = Coord.fmt,
}


local function get_entity(idx, wEntity)
	wEntity = wEntity or emu.getLabelAddress("wEntity")
	if not wEntity then
		return
	end
	local addr = wEntity.address + idx * Entity.size
	local ent = st_read(Entity, addr, wEntity.memType, string.format("$%02X", idx))
	ent.entidx = idx
	ent.isAlive = function(self)
		return self.Info & 0x80 ~= 0
	end
	return ent
end



-------- Scroll --------

Scroll = st_create("Scroll")
st_field(Scroll, "dy", 1, true)
st_field(Scroll, "y", 2)
st_field(Scroll, "row", 2)
st_field(Scroll, "y_front_dist", 1)
st_field(Scroll, "dx", 1, true)
st_field(Scroll, "x", 2)
st_field(Scroll, "column", 2)
st_field(Scroll, "x_front_dist", 1)


local fmt_scroll_pos = function(dot)
	local grid = dot >> 3
	local chunk = grid >> 4
	return string.format("%4d d | %3d g | %d c", dot, grid, chunk)
end


Scroll.fieldfmt = {
	y = fmt_scroll_pos,
	row = "%d g",
	x = fmt_scroll_pos,
	column = "%d g",
}



-------- Collide --------

Rect = st_create("Rect")
st_field(Rect, "xpos", 2)
st_field(Rect, "xend", 2)
st_field(Rect, "ypos", 2)
st_field(Rect, "yend", 2)

Rect.fieldfmt = {
	xpos = Coord.fmt,
	xend = Coord.fmt,
	ypos = Coord.fmt,
	yend = Coord.fmt,
}



-------- Emutil --------
Emutil = {}

Emutil.read = emu.read
Emutil.getLabelAddress = emu.getLabelAddress
Emutil.getState = emu.getState()

Emutil.lbRead = function(label, signed)
	return Emutil.read(label.address, label.memType, signed)
end

Emutil.readFields = function(label, spec)
	if type(label) == "string" then
		label = Emutil.getLabel(label, true)
	end
	if not label then
		Emutil.error("readFields failed.")
		return
	end
	return st_readFields(label.address, label.memType, spec)
end

Emutil.readBytes = function(startAddress, memType, length, signed)
	signed = signed or false
	local addr = startAddress
	local bytes = {}
	for _ = 1, length do
		local value = Emutil.read(addr, memType, signed)
		table.insert(bytes, value)
		addr = addr + 1
	end
	return bytes
end

Emutil.getLabel = function(symbol, failedError)
	local label = Emutil.getLabelAddress(symbol)
	if not label then
		if failedError then
			Emutil.error(string.format("Label '%s' was not found", symbol))
		end
		return nil
	end
	label.symbol = symbol
	return label
end

Emutil.getFrameCount = function()
	return Emutil.getState()["frameCount"]
end

Emutil.getScanline = function()
	return Emutil.getState()["ppu.ly"]
end

Emutil.getCpuCycleCount = function()
	return Emutil.getState()["cpu.cycleCount"]
end

Emutil.log = function(msg)
	emu.log(msg)
end

Emutil.error = function(msg)
	Emutil.log("Emutil.error: " .. msg)
	error("Emutil.error: " .. msg)
end

--- Wraps emu.isKeyPressed, adding support for modifier keys without a Left/Right specified.
Emutil.isKeyPressed = function(key)
	if key == "Shift" or key == "Ctrl" or key == "Alt" then
		return emu.isKeyPressed("Left "..key) or emu.isKeyPressed("Right "..key)
	else
		return emu.isKeyPressed(key)
	end
end

--- Check if a key combination is pressed.
--- chord: a set of key names (via ipairs(chord)) OR a string with key names separated by "+".
Emutil.isChordPressed = function(chord)
	if type(chord) == "table" then
		for _i,key in ipairs(chord) do
			if Emutil.isKeyPressed(key) == false then
				return false
			end
		end
		return true
	elseif type(chord) == "string" then
		local parts = {}
		for str in chord:gmatch("([^+]+)") do
			table.insert(parts, str)
		end
		return Emutil.isChordPressed(parts)
	else
		Emutil.error("chord should be table (list) or string")
		return false
	end
end


-------- Map Buffer Inspector --------

local MapChunkSlot = {
	SLOT_BUFFER = 0x0F,
	SLOT_RENDERED = 0x10,
	SLOT_NOCHUNK = 0x20,

	NAME = {
		"NW", "NN", "NE",
		"WW", "CC", "EE",
		"SW", "SS", "SE",
	}
}

MapChunkSlot.slotNameIndexed = function(i)
	assert(i >= 0)
	assert(i < #MapChunkSlot.NAME)
	return MapChunkSlot.NAME[i + 1]
end

MapChunkSlot.decode = function(data)
	local t = {}
	t.buffer = data & MapChunkSlot.SLOT_BUFFER
	t.rendered = data & MapChunkSlot.SLOT_RENDERED ~= 0
	t.nochunk = data & MapChunkSlot.SLOT_NOCHUNK ~= 0
	return t
end

MapChunkSlot.read = function(address, memType)
	local data = emu.read(address, memType)
	return MapChunkSlot.decode(data)
end

MapChunkSlot.readArray = function(startAddress, memType, count)
	local out = {}
	local addr = startAddress
	for _ = 1, count do
		table.insert(out, MapChunkSlot.read(addr, memType))
		addr = addr + 1
	end
	return out
end

MapTool = {
	CHUNK_SIZE = 16 * 16,
	CHUNK_SLOTS_COUNT = 9,
	SCRATCH_SLOTS_COUNT = 3,
}
MapTool.create = function()
	local bufferIdIndex = {}
	local labelsMapBufferChr = {}
	local labelsMapBufferAtrb = {}
	for i = 1, 9 do
		local label0 = Emutil.getLabel(string.format("wMapBufferChr%d", i - 1))
		if label0 then
			bufferIdIndex[(label0.address >> 8) & MapChunkSlot.SLOT_BUFFER] = i
		end
		table.insert(labelsMapBufferChr, label0)
		local label1 = Emutil.getLabel(string.format("wMapBufferAtrb%d", i - 1))
		table.insert(labelsMapBufferAtrb, label1)
	end

	local inst = {
		tile_size = 4,
		originX = 0,
		originY = 0,
		bufferIdIndex = bufferIdIndex,

		labels = {
			slots = Emutil.getLabel("hMapChunkSlots"),
			chr_buffers = labelsMapBufferChr,
			atrb_buffers = labelsMapBufferAtrb,
		},

		readState = function(self)
			if self.labels.slots then
				local l = self.labels.slots
				self.slots = MapChunkSlot.readArray(l.address, l.memType, MapTool.CHUNK_SLOTS_COUNT)
			end

			local chr_buffers = {}
			for _, label in ipairs(self.labels.chr_buffers) do
				table.insert(chr_buffers, Emutil.readBytes(label.address, label.memType, MapTool.CHUNK_SIZE))
			end
			self.chr_buffers = chr_buffers
		end,

		gridDisplayPos = function(self, column, row)
			return self.originX + column * self.tile_size, self.originY + row * self.tile_size
		end
	}
	return inst
end


-------- Map SynXfer --------
local SynXfer = st_create("SynXfer")
st_field(SynXfer, "status", 1)
st_field(SynXfer, "length", 1)
st_field(SynXfer, "destIndex", 2)
st_field(SynXfer, "srcIndex", 1)

SynXfer.fieldfmt = {
	status = function(x)
		local slotIndex = x & 0x0F
		local isEmpty = slotIndex == 0x0F
		if isEmpty then
			return "NIL"
		else
			local orientation = (x & 0x80 == 0) and "ROW" or "COL"
			local slotName = MapChunkSlot.slotNameIndexed(slotIndex)
			local sflags = ""
			if x & MapChunkSlot.SLOT_NOCHUNK ~= 0 then
				sflags = sflags .. " NC"
			end
			if x & MapChunkSlot.SLOT_RENDERED ~= 0 then
				sflags = sflags .. " R"
			end
			return string.format("%s(%s)%s", slotName, orientation, sflags)
		end
	end,
	length = "%3d",
	destIndex = "%4X",
	srcIndex = "%3d",
}



-------- Scroll Fronts --------

local function frontNorth(viewY)
	return 6 + ((viewY - 7) & 0x0F)
end

local function frontSouth(viewY)
	return 25 + ((viewY + 9) & 0x0F)
end

local function frontWest(viewX)
	-- [5..20]
	return 5 + ((viewX - 6) & 0x0F)
end

local function frontEast(viewX)
	-- [26..41]
	return 26 + ((viewX + 10) & 0x0F)
end



-------- Curtain --------

StCurtain = st_create("Curtain")
st_field(StCurtain, "status", 1)
st_field(StCurtain, "x", 1)
st_field(StCurtain, "limit", 1)
st_field(StCurtain, "destinationMode", 2)

Curtain = {
--	flags = { STEADY, POSITION, MODE_CHANGE },
	F_STEADY = 1,
	F_POSITION = 2,
	F_STATE = 3,
	F_MODE_CHANGE = 4,
	mainState = {
		"OPENING",
		"OPEN",
		"CLOSING",
		"CLOSED",
	},
}

Curtain.read = function(label)
	label = label or "hCurtain"
	local data = Emutil.readFields(label, StCurtain)
	if not data then
		Emutil.error("Curtain.read failed")
		return
	end
	local iMainState = data.status & Curtain.F_STATE
	local inst = {
		rawData = data,
		iMainState = iMainState,
		mainState = Curtain.mainState[iMainState + 1],
		modeChange = data.status & Curtain.F_MODE_CHANGE ~= 0,
		progress = 0,
	}

	if data.status & Curtain.F_STEADY ~= 0 then
		if data.status & Curtain.F_POSITION ~= 0 then
			-- CLOSING
			inst.progress = data.limit - data.x
		else
			-- OPENING
			inst.progress = -data.x
		end
	else
	end

	inst.stateText = string.format("%s (%+d)", inst.mainState, inst.progress)
	for i, field in ipairs(StCurtain) do
		local value = data[field.name]
		inst.stateText = inst.stateText .. string.format("\n%s: %s", field.name, value)
	end
	return inst
end


-------- Do stuff --------

local memTypeNames = {}
for k,v in pairs(emu.memType) do
	memTypeNames[v] = k
end

local dottogrid = function(d)
	return d >> 3
end

local gridtochunk = function(g)
	return g >> 4
end

local iMinMax = function(t, min0, max0)
	local lo = min0
	local hi = max0
	for i,x in ipairs(t) do
		lo = math.min(lo or x, x)
		hi = math.max(hi or x, x)
	end
	return lo, hi
end

local absMax = function(a, b)
	return math.abs(a) >= math.abs(b) and a or b
end

local iAbsMax = function(t, max0)
	local hi = max0
	for _, x in ipairs(t) do
		hi = absMax(hi or x, x)
	end
	return hi
end


local function frontThing(state, viewY, viewX)
	local frontN = frontNorth(viewY)
	local frontS = frontSouth(viewY)
	local frontW = frontWest(viewX)
	local frontE = frontEast(viewX)
	local xNS, northY = state.mapTool:gridDisplayPos(4, frontN)
	local _, southY = state.mapTool:gridDisplayPos(0, frontS)
	local westX, yWE = state.mapTool:gridDisplayPos(frontW, 4)
	local eastX, _ = state.mapTool:gridDisplayPos(frontE, 0)
	Overlay:drawLine(xNS, northY, 128, northY, 0x2080FF00)
	Overlay:drawLine(xNS, southY, 128, southY, 0x2000FF80)
	Overlay:drawLine(westX, yWE, westX, yWE + 128, 0x20FF8000)
	Overlay:drawLine(eastX, yWE, eastX, yWE + 128, 0x20FF0080)
end


local function scrollThing(state)
	local scroll = Scroll:readFromLabel("wScroll")
	if scroll then
		-- draw view rect on map
		local viewRow = math.floor(scroll.y / 8)
		local viewCol = math.floor(scroll.x / 8)
		local viewX, viewY = state.mapTool:gridDisplayPos(viewCol, viewRow)
		local viewDispW = 20 * state.mapTool.tile_size
		local viewDispH = 18 * state.mapTool.tile_size
		Overlay:drawRectangle2(viewX, viewY, viewDispW, viewDispH, 0x30EEEEEE)
		local viewCentreRow = 16 + ((viewRow + 9) & 0x0F)
		local viewCentreCol = 16 + ((viewCol + 10) & 0x0F)
		local viewCentreX, viewCentreY = state.mapTool:gridDisplayPos(viewCentreCol, viewCentreRow)
		Overlay:drawRectangle2(viewCentreX - viewDispW / 2, viewCentreY - viewDispH / 2, viewDispW, viewDispH, 0x30B0F030)
		frontThing(state, viewRow, viewCol)

		local px = 220
		local py = 0
		local s = st_fmt(scroll)
		local dy, dx
		if state.monScroll then
			dy = iAbsMax(state.monScroll.changes.dy, 0)
			dx = iAbsMax(state.monScroll.changes.dx, 0)
			s = s .. string.format(" dx,dy: %3d,%3d", dx, dy)
			state.monScroll:tick()
		else
			state.monScroll = st_monitor(Scroll, scroll._addr, scroll._memType)
		end
		Overlay:drawString(px, py, s)
		if dx then
			Overlay:drawVector(px + 16, py + 52 + 16, dx, dy, 15, 16, 0xEE80DD, 0xA0303030)
		end
	end
end


local function mapChunkThing(mapTool)
	-- draw chunk cache
	if mapTool.chr_buffers and #mapTool.chr_buffers == 9 then
		local chunkDispSize = 16 * mapTool.tile_size
		local _, dispH = Overlay:getSize()
		mapTool.originX = 0
		mapTool.originY = dispH - chunkDispSize * 3

		for cy = 0, 2 do
			local py = mapTool.originY + cy * chunkDispSize
			for cx = 0, 2 do
				local px = mapTool.originX + cx * chunkDispSize
				local idx = 1 + cy * 3 + cx
				local slot = mapTool.slots[idx]
				local bufferIdx = mapTool.bufferIdIndex[slot.buffer & MapChunkSlot.SLOT_BUFFER]
				local chrs = mapTool.chr_buffers[bufferIdx]
				if slot.nochunk then
					Overlay:drawRectangle(px, py, chunkDispSize, chunkDispSize, 0x10603030, true)
					Overlay:drawLine(px, py, px + chunkDispSize, py + chunkDispSize, 0xC09010)
					Overlay:drawLine(px + chunkDispSize, py, px, py + chunkDispSize, 0xC09010)
				elseif not chrs or #chrs == 0 then
					Overlay:drawRectangle(px, py, chunkDispSize, chunkDispSize, 0x10505020, true)
					Overlay:drawLine(px, py, px + chunkDispSize, py + chunkDispSize, 0xB0B010)
					Overlay:drawLine(px + chunkDispSize, py, px, py + chunkDispSize, 0xB0B010)
				else
					Overlay:drawTilemap(px, py, chrs, 16, 16, mapTool.tile_size)
				end

				local sflags = slot.rendered and "R" or "..."
				Overlay:drawString(px, py, string.format("%X %s", slot.buffer, sflags))
			end
		end
	end
end


local function drawEntityMarker(entidx)
	local scroll = Scroll:readFromLabel("wScroll")
	if not scroll then
		return
	end
	local ent = get_entity(entidx)
	if ent then
		local x, y = Overlay:oamToOverlay(Coord.units(ent.PosX) - scroll.x, Coord.units(ent.PosY) - scroll.y)
		if ent:isAlive() then
			Overlay:drawLine2(x, y, x - 8, y + 8, 0xC0A010)
			Overlay:drawLine2(x, y, x, y + 8, 0xC0A010)
		else
			Overlay:drawLine2(x - 8, y - 8, x + 8, y + 8, 0xC01010)
			Overlay:drawLine2(x - 8, y + 8, x + 8, y - 8, 0xC01010)
		end
		Overlay:drawString(x + 2, y + 2, tostring(entidx))
	end
end


local function doConfigMenu(state)
	local pointInRect = function(px, py, rx, ry, rw, rh)
		if px < rx or px >= rx + rw then
			return false
		elseif py < ry or py >= ry + rh then
			return false
		else
			return true
		end
	end
	local x = 0
	local y = 0
	local changes = {}

	-- sort config keys
	local keys = {}
	for k,_ in pairs(state.config) do
		table.insert(keys, k)
	end
	table.sort(keys)

	-- iter config items
	for _,k in ipairs(keys) do
		local v = state.config[k]
		if type(v) == "boolean" then
			local line = k..": OFF"
			local sz = emu.measureString(line)
			if v then
				line = k..": ON"
			end
			local picked = pointInRect(state.mouse.localX, state.mouse.localY, x, y, sz.width + 4, sz.height)
			local fg = Overlay.fg
			local bg = Overlay.bg
			if picked then
				fg = Overlay.bg
				bg = Overlay.fg
				if state.mouse.leftPressed then
					changes[k] = not v
				end
			end
			Overlay:drawString(x, y, line, fg, bg)
			y = y + sz.height
		end
	end

	-- apply changes
	for k,v in pairs(changes) do
		state.config[k] = v
	end
end


local function updateMouse(state)
	local mouse = Overlay:getLocalMouse()
	local buttons = { "left", "right", "middle" }
	for _i,k in ipairs(buttons) do
		local changed = mouse[k] ~= state.mouse[k]
		mouse[k.."Pressed"] = changed and mouse[k]
		mouse[k.."Released"] = changed and not mouse[k]
	end
	state.mouse = mouse
end


local function onEndFrame()
	Overlay:select()
	Overlay:drawWindow()

	updateMouse(State)
	State.mapTool:readState()

	for k,thing in pairs(Things) do
		if State.config[k] then
			thing.update(State)
		elseif State.config[k] == nil then
			State.config[k] = false
		end
	end

	if State.mouse.rightPressed then
		State.configOpen = not State.configOpen
	end

	if State.configOpen then
		doConfigMenu(State)
	end
end


Things = {}
--[[
Things.NEW = {
	update = function(state)
	end,
}
--]]
Things.WorldBounds = {
	update = function(state)
		local worldBounds = Rect:readFromLabel("wCollideBounds")
		Overlay:drawString(120, 0, worldBounds and st_fmt(worldBounds, "wCollideBounds") or "noworldBounds!")
	end,
}
Things.Scroll = {
	update = function(state)
		scrollThing(state)
	end,
}
Things.MapChunks = {
	update = function(state)
		mapChunkThing(state.mapTool)
	end,
}
Things.MapSync = {
	update = function(state)
		for i = 0, 2 do
			local sym = string.format("_Xfer%d", i)
			local xfer = SynXfer:readFromLabel(sym)
			local s = string.format("%s: NOT FOUND", sym)
			if xfer then
				s = st_fmt(xfer)
			end
			Overlay:drawString(i * 100, 200, s)
		end
	end,
}
Things.OverlayRuler = {
	update = function(state)
		local surfW,surfH = Overlay:getSurfaceSize()
		for x = 0, surfW, 32 do
			Overlay:drawString(x+2, 0, tostring(x))
			Overlay:drawLine(x, -8, x, 8, Overlay.fg)
		end
		for y = 0, surfH, 32 do
			local str = tostring(y)
			local sz = emu.measureString(str)
			Overlay:drawString(0, y+2, str)
			Overlay:drawLine(-8, y, 8, y, Overlay.fg)
		end
	end,
}
Things.Entity0 = {
	update = function(state)
		local ent = get_entity(0)
		Overlay:drawString(0, 40, ent and st_fmt(ent) or "noent!")
	end,
}
Things.EntityMarkers = {
	update = function(state)
		for i = 1, 16 do
			drawEntityMarker(i - 1)
		end
	end,
}
Things.Curtain = {
	update = function(state)
		local curtain = Curtain.read()
		if curtain then
			Overlay:drawString(40, 0, curtain.stateText)
		end
	end,
}

Overlay = createOverlay({ x = 4, y = 4 }, 2)

Config = {
	EntityMarkers = true,
}

State = {}


function GetConfig()
	return Config
end


function Setup()
	ClearState()

	State.configOpen = false
	State.config = GetConfig()

	State.mouse = Overlay:getLocalMouse()

	State.callbacks = {}
	State.addEventCallback = function(target, eventType)
		local evToken = emu.addEventCallback(target, eventType)
		table.insert(State.callbacks, { evToken = evToken, eventType = eventType, target = target })
	end

	State.mapTool = MapTool.create()

	local wScroll = Scroll:getLabelAddress("wScroll")
	if wScroll then
		State.monScroll = st_monitor(Scroll, wScroll.address, wScroll.memType)
	end

	State.doneSetup = true

	State.addEventCallback(onEndFrame, emu.eventType.endFrame)
	State.addEventCallback(function()
		ClearState()
	end, emu.eventType.reset)

	State.addEventCallback(function()
		ClearState()
	end, emu.eventType.scriptEnded)
end


function ClearState()
	if State.callbacks then
		for _,v in ipairs(State.callbacks) do
			emu.removeEventCallback(v.evToken, v.eventType)
		end
	end
	State = {}
end


--- This is the runonceatron3300
local runonceEv = nil
local function runonce()
	emu.displayMessage("runonceatron3300", "runoncening...")
	emu.removeEventCallback(runonceEv, emu.eventType.startFrame)
	runonceEv = nil
	Setup()
end
runonceEv = emu.addEventCallback(runonce, emu.eventType.startFrame)

