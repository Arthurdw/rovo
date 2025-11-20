local M = {}

-- Track if already set up to prevent duplicate autocmds
local setup_done = false

-- Debounce timers per buffer for proper debouncing
local debounce_timers = {}

-- Setup syntax highlighting for Rovo annotations
local function setup_highlighting()
  -- Get libuv handle for proper timer management
  local uv = vim.uv or vim.loop

  -- Link to standard Vim highlight groups
  vim.api.nvim_set_hl(0, 'RovoAnnotation', { link = 'Identifier', default = true })
  vim.api.nvim_set_hl(0, 'RovoStatusCode', { link = 'Number', default = true })
  vim.api.nvim_set_hl(0, 'RovoSecurityScheme', { link = 'String', default = true })

  -- Namespace for extmarks (allows flash.nvim backdrop to overlay properly)
  local ns_id = vim.api.nvim_create_namespace('rovo_highlights')

  -- Setup context-aware syntax highlighting that only applies near #[rovo]
  local function apply_rovo_highlights(bufnr)
    bufnr = bufnr or vim.api.nvim_get_current_buf()

    -- Clear existing Rovo extmarks
    vim.api.nvim_buf_clear_namespace(bufnr, ns_id, 0, -1)

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

    -- Apply highlighting using extmarks with priority below flash.nvim's backdrop (5000)
    -- Priority 4999 is high enough to prevent Rust syntax from clearing, but low enough
    -- to allow flash.nvim's backdrop to overlay and dim the highlights properly
    for _, range in ipairs(highlight_ranges) do
      local start_line, end_line = range[1], range[2]

      for line_idx = start_line, end_line do
        local line = lines[line_idx]
        if not line then break end

        -- Highlight annotation keywords (@response, @tag, etc.)
        for _, annotation in ipairs({'@response', '@tag', '@security', '@example', '@id', '@hidden'}) do
          -- Check if line is a doc comment with this annotation
          if line:match('^%s*///%s*' .. annotation:gsub('[@]', '%%@')) then
            local start_col, end_col = line:find(annotation, 1, true)
            if start_col then
              vim.api.nvim_buf_set_extmark(bufnr, ns_id, line_idx - 1, start_col - 1, {
                end_col = end_col,
                hl_group = 'RovoAnnotation',
                priority = 4999,
              })
            end
          end
        end

        -- Highlight status codes (100-599)
        local status_match = line:match('^%s*///%s*@%w+%s+(%d%d%d)')
        if status_match then
          local code = tonumber(status_match)
          if code and code >= 100 and code <= 599 then
            local start_col, end_col = line:find(status_match, 1, true)
            if start_col then
              vim.api.nvim_buf_set_extmark(bufnr, ns_id, line_idx - 1, start_col - 1, {
                end_col = end_col,
                hl_group = 'RovoStatusCode',
                priority = 4999,
              })
            end
          end
        end

        -- Highlight security schemes (bearer, basic, apiKey, oauth2)
        local security_match = line:match('^%s*///%s*@security%s+(%w+)')
        if security_match and (security_match == 'bearer' or security_match == 'basic' or
                               security_match == 'apiKey' or security_match == 'oauth2') then
          local start_col, end_col = line:find(security_match, 1, true)
          if start_col then
            vim.api.nvim_buf_set_extmark(bufnr, ns_id, line_idx - 1, start_col - 1, {
              end_col = end_col,
              hl_group = 'RovoSecurityScheme',
              priority = 4999,
            })
          end
        end
      end
    end
  end

  -- Store function globally for debugging
  _G._rovo_apply_highlights = apply_rovo_highlights

  -- Create augroup for idempotency (clear=true ensures no duplicates)
  local augroup = vim.api.nvim_create_augroup('RovoHighlighting', { clear = true })

  -- Apply on FileType
  vim.api.nvim_create_autocmd('FileType', {
    group = augroup,
    pattern = 'rust',
    callback = function(args)
      apply_rovo_highlights(args.buf)
    end,
  })

  -- Also apply when entering a Rust buffer (for already-opened files)
  vim.api.nvim_create_autocmd('BufEnter', {
    group = augroup,
    pattern = '*.rs',
    callback = function(args)
      apply_rovo_highlights(args.buf)
    end,
  })

  -- Recheck when buffer changes (in case user adds/removes #[rovo])
  vim.api.nvim_create_autocmd({'BufWritePost', 'TextChanged', 'TextChangedI'}, {
    group = augroup,
    pattern = '*.rs',
    callback = function(args)
      local bufnr = args.buf

      -- Stop and close existing timer to prevent handle leaks
      if debounce_timers[bufnr] then
        local old_timer = debounce_timers[bufnr]
        if not old_timer:is_closing() then
          old_timer:stop()
          old_timer:close()
        end
        debounce_timers[bufnr] = nil
      end

      -- Create new libuv timer (properly debounced - only last call executes)
      local timer = uv.new_timer()
      debounce_timers[bufnr] = timer

      timer:start(500, 0, function()
        -- Stop and close timer to prevent leaks
        if not timer:is_closing() then
          timer:stop()
          timer:close()
        end
        debounce_timers[bufnr] = nil

        -- Apply highlights if buffer is still valid
        if vim.api.nvim_buf_is_valid(bufnr) then
          vim.schedule(function()
            apply_rovo_highlights(bufnr)
          end)
        end
      end)
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
  -- Prevent duplicate setup to avoid creating duplicate autocmds
  if setup_done then
    return
  end
  setup_done = true

  opts = opts or {}

  -- Setup syntax highlighting
  if opts.enable_highlighting ~= false then
    setup_highlighting()
  end

  -- Ensure both LSPs can coexist (guard against missing lspconfig)
  local ok_lsp, lsp = pcall(require, 'lspconfig')
  if not ok_lsp then
    vim.notify('[rovo] nvim-lspconfig is required for LSP setup', vim.log.levels.WARN)
    return
  end

  local ok_configs, configs = pcall(require, 'lspconfig.configs')
  if not ok_configs then
    vim.notify('[rovo] Failed to load lspconfig.configs; skipping LSP setup', vim.log.levels.WARN)
    return
  end

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
