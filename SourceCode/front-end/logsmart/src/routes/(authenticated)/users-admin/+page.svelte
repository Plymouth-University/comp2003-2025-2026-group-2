<script lang="ts">
    let userTemplate = $state([
        {id:1, value: 1, first:"John", last:"Doe", role:'Admin', username:'jdoe2@organisation.org', password:'#####', dob:'10/10/2004'},
        {id:2, value: 2, first:"Jane", last:"Doe", role:'Admin', username:'jdoe3@organisation.org', password:'#####', dob:'11/02/2003'},
        {id:3, value: 3, first:"Admin", last:"Adminson", role:'User', username:'aabee124@organisation.org', password:'#####', dob:'07/04/2000'},
        {id:4, value: 4, first: 'Mai', last:'Dansfkhwerfgtrty', role:'User', username:'maidans@organisation.org', password:'#####', dob:'03/12/1972'},
        {id:5, value: 5, first: 'User', last:'Userson', role:'Admin', username:'userson@organisation.org', password:'#####', dob:'02/05/1950'}
    ]);

    function hide(){
        let state = document.getElementById("userSidebar")!.style.display;
        if (state == "flex"){
            document.getElementById("userSidebar")!.style.display = "none";
        }
    }

    function requestUser(itemid: number){
        const id = itemid;
        const userinfo = userTemplate.filter(obj => obj.id === itemid);
        // const username = userinfo.map(userinfo => userinfo.first) + " " + userinfo.map(userinfo => userinfo.last);
        let username = "" + userinfo.map(userinfo => userinfo.username);
        let first = "" + userinfo.map(userinfo => userinfo.first);
        let last = "" + userinfo.map(userinfo => userinfo.last);
        let dob = "" + userinfo.map(userinfo => userinfo.dob);
        let role = "" + userinfo.map(userinfo => userinfo.role);
        (<HTMLInputElement>document.getElementById("username")!).value = username;
        (<HTMLInputElement>document.getElementById("fname")!).value = first;
        (<HTMLInputElement>document.getElementById("lname")!).value = last;
        (<HTMLInputElement>document.getElementById("dob")!).value = dob;
        (<HTMLOptionElement>document.getElementById("userRole")!).innerHTML = role;
        let rolecheck = document.getElementById("userRole")!.innerHTML;
        if (rolecheck == "Admin"){
            (<HTMLOptionElement>document.getElementById("adminRole")!).innerHTML = "User";
        }
        else{
            (<HTMLOptionElement>document.getElementById("adminRole")!).innerHTML = "Admin";
        }
        document.getElementById("userSidebar")!.style.display = "flex";
        document.getElementById("eventHide")!.addEventListener("dblclick", hide);
        /*let state = document.getElementById("userSidebar")!.style.display;
        if (state == "none"){
            document.getElementById("userSidebar")!.style.display = "flex";
        }
        else{
            document.getElementById("userSidebar")!.style.display = "none";
        }*/
    }
</script>

<style>
    .userBox-rows{
        padding:1%;
        padding-bottom:1%;
        padding-left:5%;
        padding-right:5%;
        padding-top:1.5%;
        
    }

</style>



<main class="min-h-full" style="background-color: #F8F8F8;">
            <div class="flex h-[calc(100vh-73px)] overflow-none">
                <div class="mx-auto w-1/3 md:w-full">
                    <div class="flex-1 p-6 overflow-auto gap-1">
                        <div id="eventHide" class="flex flex-auto flex-col">
                            <!-- Display each user as a new row-->
                                {#key userTemplate}
                                    {#each userTemplate as item}
                                        <button onclick={() => requestUser(item.id)} class="flex flex-auto h-20 flex-row border-4 content-between bg-white hover:bg-gray-100 active:bg-gray-200 overflow-x-auto" style="margin-top: -5px; display:flex;">
                                            <div class="userBox-rows">{item.value}</div>
                                            <div class="userBox-rows" style="flex:1">{item.first} {item.last}</div>
                                            <div class="userBox-rows">{item.role}</div>
                                            <div class="userBox-rows"><a class="focus:underline hover:underline" href="#">Remove</a></div>
                                        </button>
                                    {/each}
                                {/key}
                                <!-- Floating action "Add New User" button-->
                                <button data-modal-target="registration-modal" style="position:relative; display:inline-block; margin-left:95%;" class="fixed z-90 bottom-10 bg-neutral-100 w-20 h-20 pt-3 border-4 border-stone-800 rounded-full drop-shadow-lg flex justify-center items-center text-black text-4xl hover:bg-gray-200 hover:drop-shadow-2xl hover:animate-bounce duration-300 cursor-pointer" type="button">&#10133;
                                    <span style="color:black; font-size:13px;">Add New</span>
                                </button>
                                <!-- Register New User Modal -->
                        </div>
                    </div>
                </div>
                    <div id="userSidebar" class="w-64 border-l-2 bg-white text-center" style="border-color: #000100; display:none;"> <!--display:none; -->
                        <div class="p-6">
                            <form>
                                <img src='SourceCode\front-end\logsmart\src\lib\assets\placeholder.png' alt="User Profile"> <!-- onerror={() => {this.src= 'SourceCode\front-end\logsmart\src\lib\assets\placeholder.png'}} -->
                                <input id="username" type="text" value="Username">
                                <input id="password" type="text" value="*******" disabled><button class="bg-gray-300 hover:bg-blue-700 text-black font-bold py-2 px-4 border rounded">Reset</button>
                                <input id="fname" type="text" value="First Name">
                                <input id="lname" type="text" value="Last Name">
                                <input id="dob" type="text" value="XX/XX/XXXX">
                                <select name="role" id="role">
                                    <option id="userRole" value="user"></option>
                                    <option id="adminRole" value="admin"></option>
                                </select>
                                <button class="bg-gray-300 hover:bg-blue-700 text-black font-bold py-2 px-4 border rounded">Save</button>
                            </form>
                        </div>
                    </div>  
            </div>
</main>