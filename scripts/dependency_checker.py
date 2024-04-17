"""
    global dependency = the ones in root/cargo.toml
    conflict = a global dependecy that is also imported specificly by a package

    This script ensure that:
        Every dependecy used by 2 or more packages are a global dependencies
        There is not unused global dependencies
        Every dependecy of every package is used at least one time

"""

import os


class DependencyList:
    def __init__(self, package):
        self.package = package
        self.specific = []
        self._global = []

    def add_specific(self, dep_name):
        self.specific.append(dep_name)

    def add_global(self, dep_name):
        self._global.append(dep_name)

    def get_specifics(self):
        return self.specific

    def get_globals(self):
        return self._global


def get_dependencies(package):
    file = ""
    if package == "":
        file = "cargo.toml"
    else:
        file = f"{package}/cargo.toml"

    dependencies = DependencyList(package)

    with open(file, "r") as f:
        found_dependencies = False

        for line in f:
            line = line.replace(" ", "").replace("\n", "")
            if line == "":
                continue

            if line.startswith("["):
                if line.endswith("dependencies]"):
                    found_dependencies = True
                else:
                    found_dependencies = False
                continue

            if line.startswith("#"):
                continue

            if not found_dependencies:
                continue

            raw_dep_name = line.split(".")[0].split("=")[0]

            if raw_dep_name == "shared":
                continue

            if ".workspace=true" in line:
                dependencies.add_global(raw_dep_name)
            else:
                dependencies.add_specific(raw_dep_name)

                # print(f"Not in workspace: {line}")

    return dependencies


def check_unused_dependency(dep_lst):
    import threading

    def check_unused_dependency_inner(dep, package):
        dep = dep.replace("-", "_")

        s1 = f"{dep}::"
        s2 = f"use {dep}"
        s3 = f"extern crate {dep}"

        if os.popen(f'rg "{s1}|{s2}|{s3}" {package}/src/').read() != "":
            return
        print(f"{package} appear to not use the {dep} dependency")

    threads = []
    for dep in dep_lst.get_specifics() + dep_lst.get_globals():
        t = threading.Thread(target=check_unused_dependency_inner,
                             args=(dep, dep_lst.package))
        t.start()

        threads.append(t)

    return threads


def check_conflict(check_name, l1, l2):
    conflicts = 0
    for i in l1:
        if i in l2:
            print(f"Conlict of {i} in {check_name}")
            conflicts += 1

    return conflicts


def ensure_global_used(global_dependencies, module_dependencies):
    threshold = 1
    unused = []
    for gdep in global_dependencies:
        count = 0
        for mdep in module_dependencies:
            if mdep == gdep:
                count += 1
        if count < threshold:
            unused.append(gdep)

    if len(unused) != 0:
        print(f"The global dependecies {unused} are used less than {threshold} times".replace("'", ""))
    else:
        print(f"Every global dependecy is used at least {threshold} time{'s'if threshold > 1 else ''}")


def display(txt, l):
    print(f"{txt}\n{l}\n".replace("'", "").replace("[", "").replace("]", ""))


client_dependencies = get_dependencies("client")
# display("Client specific dependencies", client_dependencies.get_specifics())

installer_dependencies = get_dependencies("installer")
# display("Server specific dependenciess", installer_dependencies.get_specifics())

# lib_dependencies = get_dependencies("shared")
# display("Lib specific dependencies", lib_dependencies.get_specifics())

global_dependencies = get_dependencies("")
# display("Global dependencies", global_dependencies.get_specifics())


total_conflicts = 0

# global
total_conflicts += check_conflict("client",
                                  client_dependencies.get_specifics(), global_dependencies.get_specifics())
total_conflicts += check_conflict("installer",
                                  installer_dependencies.get_specifics(), global_dependencies.get_specifics())
# total_conflicts += check_conflict("the shared lib",
#                                   lib_dependencies.get_specifics(), global_dependencies.get_specifics())

# client
total_conflicts += check_conflict("client-installer",
                                  client_dependencies.get_specifics(), installer_dependencies.get_specifics())
# total_conflicts += check_conflict("client-lib",
#                                   client_dependencies.get_specifics(), lib_dependencies.get_specifics())

# lib
# total_conflicts += check_conflict("lib-installer",
#                                   lib_dependencies.get_specifics(), installer_dependencies.get_specifics())

# installer
# empty :c koz all checks have been done


# display
if total_conflicts != 0:
    print(f"There was {total_conflicts} conflict{'s'if total_conflicts > 1 else ''}")
else:
    print("No conflicts")


# ensure that all gloal dependencies are used at least one time
ensure_global_used(global_dependencies.get_specifics(),
                   client_dependencies.get_globals()+installer_dependencies.get_globals())


for thread in check_unused_dependency(client_dependencies)+check_unused_dependency(installer_dependencies):
    thread.join()
