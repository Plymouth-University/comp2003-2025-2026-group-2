<script lang="ts">
    let userTemplate = $state([
        {id:1, value: 1, label: 'John Doe', role:'Admin'},
        {id:2, value: 2, label: 'Jane Doe', role:'Admin'},
        {id:3, value: 3, label: 'Admin Adminson', role:'User'},
        {id:4, value: 4, label: 'Mai Dansfkhwerfgtrty', role:'User'},
        {id:5, value: 5, label: 'User Userson', role:'Admin'}
    ]);
</script>

<style>
    .userBox-rows{
        padding:5%;
        
    }

</style>



<main class="flex flex-col gap-6">
    <div class="min-h-screen" style="background-color: #F8F8F8;">
        <div class="flex-1 p-6 overflow-auto gap-8">
            <div class="max-w-2xl mx-auto">
                <div class="flex flex-auto flex-col">
                    <!-- Display each user as a new row-->
                        {#key userTemplate}
                            {#each userTemplate as item}
                                <button class="flex flex-auto flex-row border-4 content-between bg-slate-50 hover:bg-gray-100 active:bg-gray-200 overflow-x-auto" style="margin-top: -2px; display:flex;">
                                    <div class="userBox-rows">{item.value}</div>
                                    <div class="userBox-rows" style="flex:1">{item.label}</div>
                                    <div class="userBox-rows">{item.role}</div>
                                    <div class="userBox-rows"><a class="focus:underline hover:underline" href="#">Remove</a></div>
                                </button>
                            {/each}
                        {/key}
                        <!-- Floating action "Add New User" button-->
                        <button data-modal-target="registration-modal" style="position:relative; display:inline-block;" class="fixed z-90 bottom-10 right-8 bg-neutral-100 w-20 h-20 pt-3 border-4 border-stone-800 rounded-full drop-shadow-lg flex justify-center items-center text-black text-4xl hover:bg-blue-100 hover:drop-shadow-2xl hover:animate-bounce duration-300 cursor-pointer" type="button">&#10133;
                            <span style="color:black; font-size:13px;">Add New</span>
                        </button>
                        <!-- Register New User Modal -->
                </div>
                <div class="flex-1 p-6 overflow-auto gap-8">
                    <h2>hiiiiii</h2> 
                </div>  
            </div>
        </div>

    </div>
</main>