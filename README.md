**This is a small piece of software to manage your key binds, or simply a personal cheatsheet for your binds in Hyprland.**

- Written in C++ (MVC pattern).
- Beta version, but works well.
- Tested on Garuda Arch Linux.

**Executable path:** `cmake-build-release/bindAnalyzer`

### How to use:

Once executed, you must have a `hyprland.conf` file in `$(HOME)/.config/hypr/`.  
`bindAnalyzer` will read your config file and extract your key binds, displaying them in a clear grid format.

### Filtering with TAGS:

You can configure **TAGS** to filter your binds.  
Add as many lines as you need in the following format:

```
\#TAGS: tag1 tag2  
bind = ...  
bind = ...  

\#TAGS: tag3 tag4  
\#Finish your tags with this:  
\#TAGS:
```

You only need **one final `#TAGS:` line** in your file.
### Filtering commands:

- **E** → Enable all tags
- **D** → Disable all tags
- **T + number + Enter** → Toggle the state of the selected tag

Use this software at your own risk.  
Distributed under the **GPL License**.