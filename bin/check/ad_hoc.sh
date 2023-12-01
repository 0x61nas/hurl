#!/bin/bash
set -Eeuo pipefail

color_red=$(echo -e "\033[1;31m")
color_green=$(echo -ne "\033[1;32m")
color_reset=$(echo -e "\033[0m")
errors_count=0

# Check *.rs Orange Copyright
echo "------------------------------------------------------------------------------------------"
while read -r rust_file ; do
    if [ "$(grep -c "Copyright (C) 2023 Orange" "$rust_file" || true)" -eq 0 ] ; then
        echo "Missing [Copyright (C) 2023 Orange] in ${color_red}${rust_file}${color_reset}"
        ((errors_count++))
    else
        echo "[Copyright (C) 2023 Orange] is present in ${color_green}${rust_file}${color_reset}"
    fi
done < <(find packages -type f -name "*.rs")

# Check *sh bash shebang at line 1
echo "------------------------------------------------------------------------------------------"
while read -r script ; do
    if [ "$(head -1 "$script" | grep -c "#!/bin/bash" || true)" -eq 0 ] ; then
        echo "Missing [#!/bin/bash] shebang in ${color_red}${script}${color_reset}"
        ((errors_count++))
    else
        echo "[#!/bin/bash] shebang is present in ${color_green}${script}${color_reset}"
    fi
done < <(find . -type f -name "*.sh")

# Check *sh error handling at first uncommented line
echo "------------------------------------------------------------------------------------------"
while read -r script ; do
    if [ "$(grep -Ev "^$|^#" "$script" | head -1 | grep -c "set -Eeuo pipefail" || true)" -eq 0 ] ; then
        echo "Missing [set -Eeuo pipefail] in ${color_red}${script}${color_reset}"
        ((errors_count++))
    else
        echo "[set -Eeuo pipefail] is present in ${color_green}${script}${color_reset}"
    fi
done < <(find . -type f -name "*.sh")

# Check bash function names in kebab case instead of camel case
echo "------------------------------------------------------------------------------------------"
while read -r script ; do
    kebab_case_function_list=$( (grep -Ev "^#" "${script}" || true) | (grep -E "^function" "${script}" || true) | (grep '-' || true) )
    if [ -n "${kebab_case_function_list}" ] ; then
        echo "Kebab case is not allowed for function naming, use snake case instead in ${color_red}${script}${color_reset}"
        while read -r function ; do
            clean_function=$(echo "${function}" | tr -s ' ' | cut --delimiter ' ' --field 2 | cut --delimiter '(' --field 1)
            echo "${color_red}${clean_function}${color_reset} have to be: $(echo "${clean_function}" | tr '-' '_')"
        done < <(echo "${kebab_case_function_list}")
        echo
        ((errors_count++))
    fi
done < <(find . -type f -name "*.sh")

# Check *PS1 error handling at two first lines
echo "------------------------------------------------------------------------------------------"
while read -r script ; do
    if [ "$(head -1 "$script" | grep -c "Set-StrictMode -Version latest" || true)" -eq 0 ] ; then
        echo "Missing [Set-StrictMode -Version latest] in first line of ${color_red}${script}${color_reset}"
        ((errors_count++))
    else
        echo "[Set-StrictMode -Version latest] is present in first line of ${color_green}${script}${color_reset}"
    fi
    if [ "$(head -2 "$script" | tail -1 | grep -c "\$ErrorActionPreference = 'Stop'" || true)" -eq 0 ] ; then
        echo "Missing [\$ErrorActionPreference = 'Stop'] in second line of ${color_red}${script}${color_reset}"
        ((errors_count++))
    else
        echo "[\$ErrorActionPreference = 'Stop'] is present in second line of ${color_green}${script}${color_reset}"
    fi
done < <(find . -type f -name "*.ps1")

# Control errors count
if [ "${errors_count}" -gt 0 ] ; then
    exit 1
fi

