local M = {}

-- Setup syntax highlighting for Rovo annotations
local function setup_highlighting()
  -- Link to standard Vim highlight groups
  vim.api.nvim_set_hl(0, 'RovoAnnotation', { link = 'Identifier' })
  vim.api.nvim_set_hl(0, 'RovoStatusCode', { link = 'Number' })
  vim.api.nvim_set_hl(0, 'RovoSecurityScheme', { link = 'String' })
  -- Types are highlighted by tree-sitter injections


  -- Setup context-aware syntax highlighting that only applies near #[rovo]
  local function apply_rovo_highlights(bufnr)
    bufnr = bufnr or vim.api.nvim_get_current_buf()

    -- Clear existing Rovo matches
    local matches = vim.fn.getmatches()
    for _, match in ipairs(matches) do
      if match.group and match.group:match('^Rovo') then
        vim.fn.matchdelete(match.id)
      end
    end

    -- Find all #[rovo] attributes and their line numbers
    local lines = vim.api.nvim_buf_get_lines(bufnr, 0, -1, false)
    local rovo_lines = {}

    for i, line in ipairs(lines) do
      if line:match('#%[%s*%w*[:%w]*rovo%s*%]') then
        table.insert(rovo_lines, i)
      end
    end

    -- If no #[rovo] found, don't apply highlighting
    if #rovo_lines == 0 then
      return
    end

    -- For each #[rovo], find the doc comment block above it
    local highlight_ranges = {}
    for _, rovo_line in ipairs(rovo_lines) do
      local start_line = rovo_line - 1
      -- Go backwards to find where doc comments start
      while start_line > 0 and lines[start_line]:match('^%s*///') do
        start_line = start_line - 1
      end
      start_line = start_line + 1 -- Adjust back to first doc comment

      -- Only add range if there are actually doc comments
      if start_line < rovo_line then
        table.insert(highlight_ranges, {start_line, rovo_line - 1})
      end
    end

    -- Apply highlighting only to specific line ranges
    for _, range in ipairs(highlight_ranges) do
      local start_line, end_line = range[1], range[2]
      local line_pattern = string.format('\\%%>%dl\\%%<%dl', start_line - 1, end_line + 2)

      -- Highlight annotation keywords (like macros: @response, @tag, etc.)
      -- Priority 10 for compatibility with plugins like Leap
      vim.fn.matchadd('RovoAnnotation', line_pattern .. '^\\s*\\/\\/\\/.*\\zs@\\(response\\|tag\\|security\\|example\\|id\\|hidden\\)\\ze', 10)

      -- Highlight status codes (100-599)
      vim.fn.matchadd('RovoStatusCode', line_pattern .. '^\\s*\\/\\/\\/ @\\w\\+\\s\\+\\zs[1-5][0-9]\\{2\\}\\ze', 10)

      -- Highlight security schemes
      vim.fn.matchadd('RovoSecurityScheme', line_pattern .. '^\\s*\\/\\/\\/ @security\\s\\+\\zs\\(bearer\\|basic\\|apiKey\\|oauth2\\)\\ze', 10)
    end
  end

  -- Store function globally for debugging
  _G._rovo_apply_highlights = apply_rovo_highlights

  -- Apply on FileType
  vim.api.nvim_create_autocmd('FileType', {
    pattern = 'rust',
    callback = function(args)
      apply_rovo_highlights(args.buf)
    end,
  })

  -- Also apply when entering a Rust buffer (for already-opened files)
  vim.api.nvim_create_autocmd('BufEnter', {
    pattern = '*.rs',
    callback = function(args)
      apply_rovo_highlights(args.buf)
    end,
  })

  -- Recheck when buffer changes (in case user adds/removes #[rovo])
  vim.api.nvim_create_autocmd({'BufWritePost', 'TextChanged', 'TextChangedI'}, {
    pattern = '*.rs',
    callback = function(args)
      -- Debounce: only check every 500ms
      vim.defer_fn(function()
        if vim.api.nvim_buf_is_valid(args.buf) then
          apply_rovo_highlights(args.buf)
        end
      end, 500)
    end,
  })
end

-- Expose for debugging
function M.debug_highlight()
  local bufnr = vim.api.nvim_get_current_buf()
  local lines = vim.api.nvim_buf_get_lines(bufnr, 0, -1, false)

  print("=== Rovo Debug Info ===")

  -- Check if tree-sitter is available
  local has_ts = pcall(require, 'nvim-treesitter')
  print(string.format("Tree-sitter available: %s", has_ts))

  -- Check highlight groups
  local hl_groups = {'RovoAnnotation', 'RovoStatusCode', 'RovoSecurityScheme'}
  for _, group in ipairs(hl_groups) do
    local hl = vim.api.nvim_get_hl(0, {name = group})
    print(string.format("%s -> %s", group, vim.inspect(hl)))
  end

  print("\nScanning for #[rovo] attributes...")
  local count = 0
  for i, line in ipairs(lines) do
    if line:match('#%[%s*%w*[:%w]*rovo%s*%]') then
      count = count + 1
      print(string.format("Found #[rovo] at line %d: %s", i, line))
    end
  end

  if count == 0 then
    print("No #[rovo] attributes found in buffer!")
  else
    print(string.format("Found %d #[rovo] attribute(s)", count))
  end

  -- Check active matches
  local matches = vim.fn.getmatches()
  local rovo_matches = 0
  for _, match in ipairs(matches) do
    if match.group and match.group:match('^Rovo') then
      rovo_matches = rovo_matches + 1
    end
  end
  print(string.format("Active Rovo matches: %d", rovo_matches))

  -- Force re-apply highlighting
  local apply_fn = _G._rovo_apply_highlights
  if apply_fn then
    apply_fn(bufnr)
    print("Re-applied highlighting")
  end
end

function M.setup(opts)
  opts = opts or {}

  -- Setup syntax highlighting
  if opts.enable_highlighting ~= false then
    setup_highlighting()
  end

  -- Ensure both LSPs can coexist
  local lsp = require('lspconfig')
  local configs = require('lspconfig.configs')

  if not configs.rovo_lsp then
    configs.rovo_lsp = {
      default_config = {
        cmd = { 'rovo-lsp' },
        filetypes = { 'rust' },
        root_dir = lsp.util.root_pattern('Cargo.toml'),
        settings = {},
      },
    }
  end

  -- Merge with user's on_attach if provided
  local user_on_attach = opts.on_attach
  opts.on_attach = function(client, bufnr)
    -- Rovo LSP should not handle semantic tokens (let rust-analyzer do that)
    client.server_capabilities.semanticTokensProvider = nil

    -- Call user's on_attach if provided
    if user_on_attach then
      user_on_attach(client, bufnr)
    end
  end

  lsp.rovo_lsp.setup(opts)
end

return M
