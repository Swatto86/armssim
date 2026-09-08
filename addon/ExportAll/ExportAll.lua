local addonName, Env = ...

local LibParse = LibStub("LibParse")

local FRAME_NAME = "ExportAllFrame"
local SLASH = "/exportall"
local SLASH_CHAT = "/exportallchat"

local function Print(msg)
    DEFAULT_CHAT_FRAME:AddMessage("|cffFFFF00ExportAll|r: " .. tostring(msg))
end

local function GenerateOutputAll()
    local character = Env.CreateCharacter()
    character:SetUnit("player")
    character:FillForExport(false, true)

    local total = #character.gear.items
    local equipped = 0
    for i = 1, 17 do
        if character.gear.items[i] then equipped = equipped + 1 end
    end
    local other = total - equipped
    local atBank = BankFrame and BankFrame:IsShown()

    local sourceLabel
    if other == 0 then
        sourceLabel = "equipped only"
    elseif atBank then
        sourceLabel = "equipped + bags + bank"
    else
        sourceLabel = "equipped + bags"
    end

    return LibParse:JSONEncode(character), total, sourceLabel
end

local frame, editBox, statusText

local function BuildFrame()
    local f = CreateFrame("Frame", FRAME_NAME, UIParent, "BackdropTemplate")
    f:SetSize(720, 480)
    f:SetPoint("CENTER")
    f:SetMovable(true)
    f:EnableMouse(true)
    f:RegisterForDrag("LeftButton")
    f:SetScript("OnDragStart", f.StartMoving)
    f:SetScript("OnDragStop", f.StopMovingOrSizing)
    f:SetResizable(true)
    f:SetBackdrop({
        bgFile = "Interface\\DialogFrame\\UI-DialogBox-Background",
        edgeFile = "Interface\\DialogFrame\\UI-DialogBox-Border",
        tile = true, tileSize = 32, edgeSize = 32,
        insets = { left = 11, right = 12, top = 12, bottom = 11 },
    })

    tinsert(UISpecialFrames, FRAME_NAME)

    local title = f:CreateFontString(nil, "OVERLAY", "GameFontNormal")
    title:SetPoint("TOP", 0, -8)
    title:SetText("ExportAll v" .. (Env.VERSION or "?"))

    statusText = f:CreateFontString(nil, "OVERLAY", "GameFontHighlightSmall")
    statusText:SetPoint("TOP", 0, -26)
    statusText:SetText("Click Generate to export.")

    local genBtn = CreateFrame("Button", nil, f, "UIPanelButtonTemplate")
    genBtn:SetSize(120, 24)
    genBtn:SetPoint("TOPLEFT", 16, -46)
    genBtn:SetText("Generate")
    genBtn:SetScript("OnClick", function()
        local json, count, sourceLabel = GenerateOutputAll()
        editBox:SetText(json or "")
        editBox:HighlightText()
        editBox:SetFocus()
        statusText:SetText(("Generated %d items (%s). Ctrl+C to copy."):format(count, sourceLabel))
    end)

    local close = CreateFrame("Button", nil, f, "UIPanelButtonTemplate")
    close:SetSize(80, 24)
    close:SetPoint("TOPRIGHT", -16, -46)
    close:SetText("Close")
    close:SetScript("OnClick", function() f:Hide() end)

    local clear = CreateFrame("Button", nil, f, "UIPanelButtonTemplate")
    clear:SetSize(80, 24)
    clear:SetPoint("TOP", 0, -46)
    clear:SetText("Clear")
    clear:SetScript("OnClick", function()
        editBox:SetText("")
        statusText:SetText("Click Generate to export.")
    end)

    local eb = CreateFrame("EditBox", nil, f)
    eb:SetMultiLine(true)
    eb:SetAutoFocus(false)
    eb:SetFontObject(ChatFontNormal)
    eb:SetPoint("TOPLEFT", 16, -78)
    eb:SetPoint("BOTTOMRIGHT", -16, 16)
    eb:SetScript("OnEscapePressed", function() f:Hide() end)
    eb:SetScript("OnEditFocusGained", function(self) self:HighlightText() end)

    f:Hide()
    return f, eb
end

local function EnsureFrame()
    if not frame then
        frame, editBox = BuildFrame()
    end
    return frame, editBox
end

local function OpenAndGenerate()
    local f, eb = EnsureFrame()
    f:Show()
    local json, count, sourceLabel = GenerateOutputAll()
    eb:SetText(json or "")
    eb:HighlightText()
    eb:SetFocus()
    statusText:SetText(("Generated %d items (%s). Ctrl+C to copy."):format(count, sourceLabel))
end

local function DumpToChat()
    local json, count, sourceLabel = GenerateOutputAll()
    Print(("Generated %d items (%s). Length: %d chars."):format(count, sourceLabel, #json))
    DEFAULT_CHAT_FRAME:AddMessage(json)
end

SLASH_EXPORTALL1 = SLASH
SlashCmdList["EXPORTALL"] = function(msg)
    msg = (msg or ""):lower():trim()
    if msg == "chat" then
        DumpToChat()
    else
        OpenAndGenerate()
    end
end

SLASH_EXPORTALLCHAT1 = SLASH_CHAT
SlashCmdList["EXPORTALLCHAT"] = DumpToChat

Print(("v%s loaded. |cffFFFFFF%s|r opens a frame; |cffFFFFFF%s|r dumps to chat."):format(
    Env.VERSION or "?", SLASH, SLASH_CHAT))
