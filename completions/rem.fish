# Print an optspec for argparse to handle cmd's options that are independent of any subcommand.
function __fish_rem_global_optspecs
	string join \n h/help V/version
end

function __fish_rem_needs_command
	# Figure out if the current invocation already has a command.
	set -l cmd (commandline -opc)
	set -e cmd[1]
	argparse -s (__fish_rem_global_optspecs) -- $cmd 2>/dev/null
	or return
	if set -q argv[1]
		# Also print the command, so this can be used to figure out what it is.
		echo $argv[1]
		return 1
	end
	return 0
end

function __fish_rem_using_subcommand
	set -l cmd (__fish_rem_needs_command)
	test -z "$cmd"
	and return 1
	contains -- $cmd[1] $argv
end

complete -c rem -n "__fish_rem_needs_command" -s h -l help -d 'Print help'
complete -c rem -n "__fish_rem_needs_command" -s V -l version -d 'Print version'
complete -c rem -n "__fish_rem_needs_command" -f -a "new" -d 'Add a new todo'
complete -c rem -n "__fish_rem_needs_command" -f -a "toggle" -d 'Toggle the done state of a todo by its index'
complete -c rem -n "__fish_rem_needs_command" -f -a "pending" -d 'Lists pending todos (default)'
complete -c rem -n "__fish_rem_needs_command" -f -a "all" -d 'List all todos'
complete -c rem -n "__fish_rem_needs_command" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c rem -n "__fish_rem_using_subcommand new" -l due -d 'The due date of the todo (for scheduled todos), valid format: YYYY-MM-DD' -r
complete -c rem -n "__fish_rem_using_subcommand new" -s d -l daily -d 'make the todo a daily todo'
complete -c rem -n "__fish_rem_using_subcommand new" -s h -l help -d 'Print help'
complete -c rem -n "__fish_rem_using_subcommand toggle" -s h -l help -d 'Print help'
complete -c rem -n "__fish_rem_using_subcommand pending" -s h -l help -d 'Print help'
complete -c rem -n "__fish_rem_using_subcommand all" -s h -l help -d 'Print help'
complete -c rem -n "__fish_rem_using_subcommand help; and not __fish_seen_subcommand_from new toggle pending all help" -f -a "new" -d 'Add a new todo'
complete -c rem -n "__fish_rem_using_subcommand help; and not __fish_seen_subcommand_from new toggle pending all help" -f -a "toggle" -d 'Toggle the done state of a todo by its index'
complete -c rem -n "__fish_rem_using_subcommand help; and not __fish_seen_subcommand_from new toggle pending all help" -f -a "pending" -d 'Lists pending todos (default)'
complete -c rem -n "__fish_rem_using_subcommand help; and not __fish_seen_subcommand_from new toggle pending all help" -f -a "all" -d 'List all todos'
complete -c rem -n "__fish_rem_using_subcommand help; and not __fish_seen_subcommand_from new toggle pending all help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
