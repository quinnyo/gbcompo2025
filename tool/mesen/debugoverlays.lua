
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
		emu.drawString(ox + x, oy + y, text, textColor, backgroundColor, maxWidth, duration, delay)
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
	return ent
end



-------- Scroll --------

Scroll = st_create("Scroll")
st_field(Scroll, "dy", 1, true)
st_field(Scroll, "y", 2)
st_field(Scroll, "frontier_row", 1)
st_field(Scroll, "fn_render_map_rows", 2)
st_field(Scroll, "dx", 1, true)
st_field(Scroll, "x", 2)
st_field(Scroll, "frontier_column", 1)
st_field(Scroll, "fn_render_map_columns", 2)


local fmt_scroll_pos = function(dot)
	local grid = dot >> 3
	local chunk = grid >> 4
	return string.format("%4d d | %3d g | %d c", dot, grid, chunk)
end


Scroll.fieldfmt = {
	y = fmt_scroll_pos,
	frontier_row = "%d",
	x = fmt_scroll_pos,
	frontier_column = "%d",
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



-------- Map Buffer Inspector --------

local MapChunkSlot = {
	SLOT_BUFFER = 0x0F,
	SLOT_RENDERED = 0x10,
	SLOT_NOCHUNK = 0x20,
}

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

MapChunkSlot.readArray = function(address, memType, count)
	local out = {}
	for _ = 1, count do
		table.insert(out, MapChunkSlot.read(address, memType))
	end
	return out
end

local function readBytes(startAddress, memType, length, signed)
	signed = signed or false
	local addr = startAddress
	local bytes = {}
	for _ = 1, length do
		local value = emu.read(addr, memType, signed)
		table.insert(bytes, value)
		addr = addr + 1
	end
	return bytes
end

local function getLabel(symbol)
	local label = emu.getLabelAddress(symbol)
	if not label then
		return nil
	end
	label.symbol = symbol
	return label
end

local labelsMapBufferChr = {}
local labelsMapBufferAtrb = {}
for i = 0, 8 do
	table.insert(labelsMapBufferChr, getLabel(string.format("wMapBufferChr%d", i)))
	table.insert(labelsMapBufferAtrb, getLabel(string.format("wMapBufferAtrb%d", i)))
end

MapTool = {
	CHUNK_SIZE = 16 * 16,
	CHUNK_SLOTS_COUNT = 9,
	SCRATCH_SLOTS_COUNT = 3,

	labels = {
		slots = getLabel("hMapChunkSlots"),
		chr_buffers = labelsMapBufferChr,
		atrb_buffers = labelsMapBufferAtrb,
	},

	readState = function(self)
		if self.labels.slots then
			local l = self.labels.slots
			self.slots = MapChunkSlot.readArray(l.address, l.memType, self.CHUNK_SLOTS_COUNT)
		end

		local chr_buffers = {}
		for _, label in ipairs(self.labels.chr_buffers) do
			table.insert(chr_buffers, readBytes(label.address, label.memType, self.CHUNK_SIZE))
		end
		self.chr_buffers = chr_buffers
	end,
}



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


local overlay = createOverlay({ x = 12, y = 12 }, 4)

local wScroll = Scroll:getLabelAddress("wScroll")
local monScroll = st_monitor(Scroll, wScroll.address, wScroll.memType)


local function onEndFrame()
	overlay:select()
	overlay:drawWindow()

	MapTool:readState()
	-- draw chunk cache
	if MapTool.chr_buffers and #MapTool.chr_buffers == 9 then
		local tile_size = 3
		local chunkDispSize = 16 * tile_size
		local _, dispH = overlay:getSize()
		local originX = 0
		local originY = dispH - chunkDispSize * 3
		for cy = 0, 2 do
			local py = originY + cy * chunkDispSize
			for cx = 0, 2 do
				local px = originX + cx * chunkDispSize
				local idx = 1 + cy * 3 + cx
				local chrs = MapTool.chr_buffers[idx]
				overlay:drawTilemap(px, py, chrs, 16, 16, tile_size)
			end
		end
	end

--	local surfW = overlay:getSurfaceSize()
--	for x = 0, surfW, 64 do
--		overlay:drawString(x, -8, tostring(x))
--	end

	local scroll = Scroll:readFromLabel("wScroll")
	if scroll then
		local px = 220
		local py = 0
		local s = st_fmt(scroll)
		local dy, dx
		if monScroll then
			dy = iAbsMax(monScroll.changes.dy, 0)
			dx = iAbsMax(monScroll.changes.dx, 0)
			s = s .. string.format(" dx,dy: %3d,%3d", dx, dy)
			monScroll:tick()
		end
		overlay:drawString(px, py, s)
		if dx then
			overlay:drawVector(px + 16, py + 52 + 16, dx, dy, 15, 16, 0xEE80DD, 0xA0303030)
		end
	end

	local worldBounds = Rect:readFromLabel("wCollideBounds")
	overlay:drawString(120, 0, worldBounds and st_fmt(worldBounds, "wCollideBounds") or "noworldBounds!")

	local ent = get_entity(0)
	overlay:drawString(0, 40, ent and st_fmt(ent) or "noent!")
end


emu.displayMessage("Something", "Hello.")
emu.addEventCallback(onEndFrame, emu.eventType.endFrame);

