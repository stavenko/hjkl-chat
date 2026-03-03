# Penpot Organization

## Page Structure

Each page in Penpot represents a complete workflow (user flow). All frames (boards) within a page belong to the same flow.


### Pages

- **Main** - Page, contains all frames for this application.

## Frame Layout

### X/Y Organization

- Frames in the same flow share the same Y coordinate (visual grouping)
- Frames in different flows are separated by Y coordinate (minimum 200px gap)
- Within a flow, frames are arranged horizontally with X coordinate (minimum 100px gap)

### Current Layout

```
Main Page:
  Y=0:   [1 - Enter Email]    [2 - Verify Email]    [3 - Set Password]
  Y=1052: [Login]
```

All frames in the same row represent steps in the registration flow.

## Frame Naming

Each frame has a descriptive name:
- `1 - Enter Email` - Step 1 of registration
- `2 - Verify Email` - Step 2 of registration  
- `3 - Set Password` - Step 3 of registration

## Flows

### Authentication Flow (Main Page)
1. Registration: `1 - Enter Email` → `2 - Verify Email` → `3 - Set Password` (Y=0)
2. Login: `Login` (Y=1052)
3. Logout: `Logout` (to be created)
4. Password Change: `Password Change` (to be created)

### Password Restore Flow (Password Restore Page)
1. Request password reset
2. Verify code
3. Set new password

## Dimensions

- Frame width: 393px
- Frame height: 852px (mobile-first design)
- Gap between frames in same flow: 100px
- Gap between different flows: 200px

## MCP Server

### Available Tools

| Tool | Purpose |
| ---- | ------- |
| `mcp__penpot__execute_code` | Run JavaScript in Penpot plugin context to create/modify designs |
| `mcp__penpot__export_shape` | Export shapes as PNG/SVG for visual inspection |
| `mcp__penpot__import_image` | Import images (icons, photos, logos) into designs |
| `mcp__penpot__penpot_api_info` | Retrieve Penpot API documentation |

### Setup (If Not Connected)

1. **Clone and install the MCP server:**
   ```bash
   git clone https://github.com/penpot/penpot-mcp.git
   cd penpot-mcp
   npm install
   npm run bootstrap
   ```

2. **Start the servers:**
   ```bash
   npm run start:all
   ```

3. **In Penpot UI:**
   - Open a design file
   - Go to **Plugins** → **Load plugin from URL**
   - Enter: `http://localhost:4400/manifest.json`
   - Click **"Connect to MCP server"** in the plugin UI

### Troubleshooting

| Issue | Solution |
| ----- | -------- |
| Plugin won't connect | Check servers are running (`npm run start:all` in penpot-mcp dir) |
| Browser blocks localhost | Allow local network access prompt, or disable Brave Shield, or try Firefox |
| Tools not appearing in client | Restart OpenCode completely after config changes |
| Tool execution fails/times out | Ensure Penpot plugin UI is open and shows "Connected" |
| "WebSocket connection failed" | Check firewall allows ports 4400, 4401, 4402 |

### Key API Gotchas

- `width`/`height` are READ-ONLY → use `shape.resize(w, h)`
- `parentX`/`parentY` are READ-ONLY → use `penpotUtils.setParentXY(shape, x, y)`
- Use `insertChild(index, shape)` for z-ordering (not `appendChild`)
- Flex children array order is REVERSED for `dir="column"` or `dir="row"`
- After `text.resize()`, reset `growType` to `"auto-width"` or `"auto-height"`

### Create New Board (Frame)

```javascript
// Find all existing boards and calculate next position
const boards = penpotUtils.findShapes(s => s.type === 'board', penpot.root);
let nextX = 0;
const gap = 100; // Space between boards

if (boards.length > 0) {
  boards.forEach(b => {
    const rightEdge = b.x + b.width;
    if (rightEdge + gap > nextX) {
      nextX = rightEdge + gap;
    }
  });
}

// Create new board at calculated position
const newBoard = penpot.createBoard();
newBoard.x = nextX;
newBoard.y = 0;
newBoard.resize(393, 852);
```
