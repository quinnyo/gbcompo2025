
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

	obj.drawWindow = function(self)
		local w, h = self:getSize()
		self:drawRectangle(0, 0, w, h, 0x0F9BABEB, false)
	end
	obj.getSurfaceSize = function(self)
		return 160 * self.scale, 144 * self.scale
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
	t[name] = t._ofs
	t._ofs = t._ofs + size
	t.size = t._ofs
end


local function st_read(st, addr, memType, ident)
	local data = {}
	data._st = st
	data._addr = addr
	data._memType = memType
	data._ident = ident
	for i,field in ipairs(st) do
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


local function st_create(name)
	local st = {}
	st.name = name or "Struct"
	st.read = function(addr, memType)
		return st_read(st, addr, memType)
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
		for i,field in ipairs(st) do
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

Scroll.fieldfmt = {
	y = "%d",
	x = "%d",
}



-------- Do stuff --------

local overlay = createOverlay({ x = 12, y = 12 }, 4)


local function onEndFrame()
	overlay:select()
	overlay:drawWindow()

	local wScroll = emu.getLabelAddress("wScroll")
	local scroll = st_read(Scroll, wScroll.address, wScroll.memType)
	overlay:drawString(96, 0, scroll and st_fmt(scroll) or "noscroll!")

	local wEntity = emu.getLabelAddress("wEntity")
	local ent = get_entity(0, wEntity)
	overlay:drawString(0, 40, ent and st_fmt(ent) or "noent!")
end


emu.displayMessage("Something", "Hello.")
emu.addEventCallback(onEndFrame, emu.eventType.endFrame);

