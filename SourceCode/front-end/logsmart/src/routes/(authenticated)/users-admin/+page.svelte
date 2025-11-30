<script lang="ts">

    let userTemplate = $state([
        {id:1, value: '11102', first:"John", last:"Doe", role:'Admin', username:'jdoe2@organisation.org', password:'#####', dob:'10/10/2004'},
        {id:2, value: '22331', first:"Jane", last:"Doe", role:'Admin', username:'jdoe3@organisation.org', password:'#####', dob:'11/02/2003'},
        {id:3, value: '33161', first:"Admin", last:"Adminson", role:'User', username:'aabee124@organisation.org', password:'#####', dob:'07/04/2000'},
        {id:4, value: '47145', first: 'Mai', last:'Dansfkhwerfgtrty', role:'User', username:'maidans@organisation.org', password:'#####', dob:'03/12/1972'},
        {id:5, value: '51503', first: 'User', last:'Userson', role:'Admin', username:'userson@organisation.org', password:'#####', dob:'02/05/1950'}
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
        // const fullname = userinfo.map(userinfo => userinfo.first) + " " + userinfo.map(userinfo => userinfo.last);
        let username = String(userinfo.map(userinfo => userinfo.username));
        let first = String(userinfo.map(userinfo => userinfo.first));
        let last = String(userinfo.map(userinfo => userinfo.last));
        let dob = String(userinfo.map(userinfo => userinfo.dob));
        let role = String(userinfo.map(userinfo => userinfo.role));
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
                                <div id="registration-modal" tabindex="-1" aria-hidden="true" class="hidden overflow-y-auto overflow-x-hidden fixed top-0 right-0 left-0 z-50 justify-center items-center w-full md:inset-0 h-[calc(100%-1rem)] max-h-full">
                                    <div class="relative p-4 w-full max-w-md max-h-full">
                                        <div class="relative bg-neutral-primary-soft border border-default rounded-base shadow-sm p-4 md:p-6">
                                            <div class="flex items-center justify-between border-b border-default pb-4 md:pb-5">
                                                <svg class="w-5 h-5" aria-hidden="true" xmlns="http://www.w3.org/2000/svg" width="24" height="24" fill="none" viewBox="0 0 24 24"><path stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18 17.94 6M18 18 6.06 6"/></svg>
                                                <h3 class="text-lg font-medium text-heading">REGISTER NEW USER</h3>
                                                <button type="button" class="text-body bg-transparent hover:bg-neutral-tertiary hover:text-heading rounded-base text-sm w-9 h-9 ms-auto inline-flex justify-center items-center" data-modal-hide="authentication-modal">
                                                    <svg class="w-5 h-5" aria-hidden="true" xmlns="http://www.w3.org/2000/svg" width="24" height="24" fill="none" viewBox="0 0 24 24"><path stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18 17.94 6M18 18 6.06 6"/></svg>
                                                    <span class="sr-only">Close modal</span>
                                                </button>
                                            </div>
                                            <form action="#" class="pt-4 md:pt-6">
                                                <div class="mb-4">
                                                    <label for="email" class="block mb-2.5 text-sm font-medium text-heading">New user's email</label>	
                                                    <input type="email" id="email" class="bg-neutral-secondary-medium border border-default-medium text-heading text-sm rounded-base focus:ring-brand focus:border-brand block w-full px-3 py-2.5 shadow-xs placeholder:text-body" placeholder="example@company.com" required />
                                                </div>
                                                <button aria-label="Send email">
                                                    <svg class="fill-current w-4 h-4 mr-2" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20"></svg>
                                                </button>
                                            </form>
                                        </div>
                                    </div>
                                </div>
                    
                        <!--end modal-->
                        </div>
                    </div>
                </div>
                    <div id="userSidebar" class="w-70 border-l-2 bg-white text-center" style="border-color: #000100; display:none;"> <!--display:none; -->
                        <div class="flex flex-col justify-items-center">
                            <form class="px-8 pt-2 pb-8 mb-4">
                                <img src='src\lib\assets\placeholder.png' alt="User Profile"> <!-- onerror={() => {this.src= 'SourceCode\front-end\logsmart\src\lib\assets\placeholder.png'}} -->
                                <input class="mb-2" id="username" type="text" value="Username" required>
                                <div class="flex flex-col md:flex-row gap-4">
                                    <input class="mb-2 cursor-not-allowed bg-gray-200 select-none" id="password" type="text" value="*******" style="width:60%; -webkit-user-select: none; user-select: none; -ms-user-select: none;" disabled>
                                    <button class="bg-gray-300 hover:bg-slate-700 mb-2 hover:text-white text-black font-bold py-2 px-4 border rounded cursor-pointer">Reset</button>
                                </div>
                                <input class="mb-2" id="fname" type="text" value="First Name" required>
                                <input class="mb-2" id="lname" type="text" value="Last Name" required>
                                <input class="mb-2" id="dob" type="text" value="XX/XX/XXXX" required>
                                <select class="mb-3" name="role" id="role">
                                    <option id="userRole" value="user"></option>
                                    <option id="adminRole" value="admin"></option>
                                </select>
                                <button class="m-5 mb-0 bg-gray-300 hover:bg-slate-700 hover:text-white text-black font-bold py-2 px-4 border rounded cursor-pointer">Save</button>
                            </form>
                        </div>
                    </div>  
            </div>
</main>