function kepler_widget --description 'Search commands with Kepler'
    set --local original_buffer (commandline)
    set --local original_cursor (commandline --cursor)
    set --local selection (command kepler-cli)
    set --local kepler_status $status

    if test $kepler_status -eq 0; and test (count $selection) -eq 1
        commandline --replace -- $selection[1]
        commandline --cursor (string length -- $selection[1])
    else
        commandline --replace -- $original_buffer
        commandline --cursor $original_cursor
    end

    commandline --function repaint
end
